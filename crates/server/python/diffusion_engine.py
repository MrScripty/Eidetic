"""Diffusion LLM inference engine for TraDo-8B.

Implements discrete diffusion denoising with semi-autoregressive
block-level decoding. The model processes blocks of tokens sequentially,
denoising tokens within each block in parallel across multiple steps.

Thread requirements:
- All methods must be called from the same thread that created the instance.
- The owning thread must hold the Python GIL.
- GPU tensors are not thread-safe -- do not share across threads.

Reference: TraDo-8B (Gen-Verse/dLLM-RL)
  Paper: arXiv:2509.06949
  License: MIT
"""

from __future__ import annotations

from typing import Generator

import torch
import torch.nn.functional as F


# TraDo-8B mask token ID (from tokenizer vocabulary).
MASK_TOKEN_ID = 151669

# Default semi-autoregressive block size.
DEFAULT_BLOCK_LENGTH = 4

# Default denoising steps per block.
DEFAULT_STEPS_PER_BLOCK = 4

# Confidence threshold for unmasking tokens.
DEFAULT_DYNAMIC_THRESHOLD = 0.9


class DiffusionEngine:
    """Manages model lifecycle and runs diffusion-based text infilling.

    Invariants:
    - model and tokenizer are either both None or both loaded.
    - infill() may only be called when is_loaded() is True.
    """

    def __init__(self) -> None:
        self.model: torch.nn.Module | None = None
        self.tokenizer = None
        self._device: str = "cpu"
        self._model_path: str | None = None

    def load_model(self, model_path: str, device: str = "cuda") -> None:
        """Load TraDo-8B weights into VRAM. Idempotent -- reloads if path differs.

        Args:
            model_path: HuggingFace model ID or local path to safetensors.
            device: Target device ("cuda" or "cpu").
        """
        if self.model is not None and self._model_path == model_path:
            return

        # Unload previous model first (symmetric with load).
        self.unload_model()

        from transformers import AutoModelForCausalLM, AutoTokenizer

        self.tokenizer = AutoTokenizer.from_pretrained(
            model_path, trust_remote_code=True
        )
        self.model = AutoModelForCausalLM.from_pretrained(
            model_path,
            trust_remote_code=True,
            torch_dtype=torch.float16,
            device_map=device,
        )
        self.model.eval()
        self._device = device
        self._model_path = model_path

    def unload_model(self) -> None:
        """Release model from VRAM. Symmetric with load_model()."""
        if self.model is not None:
            del self.model
            self.model = None
        if self.tokenizer is not None:
            del self.tokenizer
            self.tokenizer = None
        self._model_path = None
        if torch.cuda.is_available():
            torch.cuda.empty_cache()

    def is_loaded(self) -> bool:
        """Check whether model and tokenizer are ready."""
        return self.model is not None and self.tokenizer is not None

    def infill(
        self,
        prefix: str,
        suffix: str,
        mask_count: int,
        steps_per_block: int = DEFAULT_STEPS_PER_BLOCK,
        block_length: int = DEFAULT_BLOCK_LENGTH,
        temperature: float = 1.0,
        dynamic_threshold: float = DEFAULT_DYNAMIC_THRESHOLD,
    ) -> Generator[tuple[int, int, str], None, None]:
        """Run diffusion infilling between prefix and suffix.

        Yields (current_step, total_steps, current_text) after each
        denoising step so the caller can stream progress.

        Preconditions:
        - is_loaded() must be True.
        - mask_count > 0.
        - steps_per_block > 0.

        Algorithm: Semi-autoregressive block diffusion (TraDo)
        1. Tokenize: prefix_ids + [MASK]*mask_count + suffix_ids
        2. Divide the masked region into blocks of block_length tokens.
        3. For each block (left to right):
           a. Run steps_per_block denoising iterations on that block.
           b. Each iteration: forward pass, confidence scoring, unmask
              tokens above dynamic_threshold.
        4. yield progress after every denoising step.

        Args:
            prefix: Text before the masked region.
            suffix: Text after the masked region.
            mask_count: Number of mask tokens to insert between prefix and suffix.
            steps_per_block: Denoising iterations per block.
            block_length: Tokens per semi-autoregressive block.
            temperature: Sampling temperature (higher = more random).
            dynamic_threshold: Confidence threshold for unmasking.

        Yields:
            (current_step, total_steps, current_text) per denoising step.
        """
        assert self.model is not None and self.tokenizer is not None

        prefix_ids = self.tokenizer.encode(prefix, add_special_tokens=False)
        suffix_ids = self.tokenizer.encode(suffix, add_special_tokens=False)

        # Build the full sequence: prefix + masks + suffix.
        mask_ids = [MASK_TOKEN_ID] * mask_count
        input_ids = prefix_ids + mask_ids + suffix_ids
        seq = torch.tensor([input_ids], dtype=torch.long, device=self._device)

        # Region boundaries within the sequence.
        mask_start = len(prefix_ids)
        mask_end = mask_start + mask_count

        # Compute total steps for progress reporting.
        num_blocks = (mask_count + block_length - 1) // block_length
        total_steps = num_blocks * steps_per_block
        current_step = 0

        # Process each block left-to-right (semi-autoregressive).
        for block_idx in range(num_blocks):
            block_start = mask_start + block_idx * block_length
            block_end = min(block_start + block_length, mask_end)

            for step in range(steps_per_block):
                seq = self._denoise_step(
                    seq,
                    block_start,
                    block_end,
                    temperature,
                    dynamic_threshold,
                )

                current_step += 1

                # Decode current full sequence (skip special tokens).
                decoded = self.tokenizer.decode(
                    seq[0, mask_start:mask_end].tolist(),
                    skip_special_tokens=True,
                )
                yield (current_step, total_steps, decoded)

        # Force-unmask any remaining mask tokens in the region.
        remaining_mask_positions = (
            seq[0, mask_start:mask_end] == MASK_TOKEN_ID
        ).nonzero(as_tuple=True)[0]

        if len(remaining_mask_positions) > 0:
            seq = self._force_unmask(seq, mask_start, mask_end)
            decoded = self.tokenizer.decode(
                seq[0, mask_start:mask_end].tolist(),
                skip_special_tokens=True,
            )
            yield (total_steps, total_steps, decoded)

    def generate(
        self,
        prompt: str,
        gen_length: int = 200,
        steps_per_block: int = DEFAULT_STEPS_PER_BLOCK,
        block_length: int = DEFAULT_BLOCK_LENGTH,
        temperature: float = 1.0,
        dynamic_threshold: float = DEFAULT_DYNAMIC_THRESHOLD,
    ) -> Generator[tuple[int, int, str], None, None]:
        """Generate text continuation (prefix-only, no suffix).

        Same as infill() but with an empty suffix.

        Args:
            prompt: Text prefix to continue from.
            gen_length: Number of tokens to generate.
            steps_per_block: Denoising iterations per block.
            block_length: Tokens per semi-autoregressive block.
            temperature: Sampling temperature.
            dynamic_threshold: Confidence threshold.

        Yields:
            (current_step, total_steps, current_text) per denoising step.
        """
        yield from self.infill(
            prefix=prompt,
            suffix="",
            mask_count=gen_length,
            steps_per_block=steps_per_block,
            block_length=block_length,
            temperature=temperature,
            dynamic_threshold=dynamic_threshold,
        )

    @torch.no_grad()
    def _denoise_step(
        self,
        seq: torch.Tensor,
        block_start: int,
        block_end: int,
        temperature: float,
        dynamic_threshold: float,
    ) -> torch.Tensor:
        """Run one denoising step on the specified block.

        Predicts tokens for masked positions in [block_start, block_end),
        then unmasks positions where confidence exceeds dynamic_threshold.
        """
        # Forward pass through the full sequence.
        outputs = self.model(input_ids=seq)
        logits = outputs.logits  # (batch=1, seq_len, vocab_size)

        # Extract logits for the current block.
        block_logits = logits[0, block_start:block_end, :]  # (block_len, vocab)

        # Find which positions in this block are still masked.
        block_tokens = seq[0, block_start:block_end]
        masked_positions = (block_tokens == MASK_TOKEN_ID).nonzero(as_tuple=True)[0]

        if len(masked_positions) == 0:
            return seq

        # Compute probabilities and sample tokens for masked positions.
        masked_logits = block_logits[masked_positions]  # (n_masked, vocab)

        if temperature > 0:
            probs = F.softmax(masked_logits / temperature, dim=-1)
            sampled = torch.multinomial(probs, num_samples=1).squeeze(-1)
        else:
            sampled = masked_logits.argmax(dim=-1)

        # Compute confidence as max probability for each position.
        with torch.no_grad():
            confidence = F.softmax(masked_logits, dim=-1).max(dim=-1).values

        # Unmask positions where confidence exceeds threshold.
        high_confidence = confidence >= dynamic_threshold
        if high_confidence.any():
            absolute_positions = block_start + masked_positions[high_confidence]
            seq[0, absolute_positions] = sampled[high_confidence]

        # On the last denoising step, unmask everything remaining.
        # (Handled by the caller after the step loop.)

        return seq

    @torch.no_grad()
    def _force_unmask(
        self,
        seq: torch.Tensor,
        mask_start: int,
        mask_end: int,
    ) -> torch.Tensor:
        """Force-unmask any remaining mask tokens by argmax prediction."""
        outputs = self.model(input_ids=seq)
        logits = outputs.logits[0, mask_start:mask_end, :]

        region_tokens = seq[0, mask_start:mask_end]
        still_masked = (region_tokens == MASK_TOKEN_ID).nonzero(as_tuple=True)[0]

        if len(still_masked) > 0:
            predicted = logits[still_masked].argmax(dim=-1)
            seq[0, mask_start + still_masked] = predicted

        return seq
