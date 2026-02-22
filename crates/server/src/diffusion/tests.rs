//! Unit tests for diffusion module types and validation.

#[cfg(test)]
mod tests {
    use tokio::sync::oneshot;

    use crate::diffusion::types::*;

    #[test]
    fn test_diffusion_error_display_model_not_loaded_includes_help() {
        let err = DiffusionError::ModelNotLoaded;
        let msg = err.to_string();
        assert!(
            msg.contains("POST /ai/diffusion/load"),
            "error message should guide the user: {msg}"
        );
    }

    #[test]
    fn test_diffusion_error_display_load_failed_includes_detail() {
        let err = DiffusionError::LoadFailed("out of VRAM".into());
        let msg = err.to_string();
        assert!(msg.contains("out of VRAM"), "should include detail: {msg}");
    }

    #[test]
    fn test_diffusion_status_serializes_correctly() {
        let status = DiffusionStatus {
            model_loaded: true,
            model_path: Some("/models/trado".into()),
            device: "cuda".into(),
        };
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["model_loaded"], true);
        assert_eq!(json["model_path"], "/models/trado");
        assert_eq!(json["device"], "cuda");
    }

    #[test]
    fn test_diffusion_status_serializes_when_no_model() {
        let status = DiffusionStatus {
            model_loaded: false,
            model_path: None,
            device: "cpu".into(),
        };
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["model_loaded"], false);
        assert!(json["model_path"].is_null());
    }

    #[test]
    fn test_diffuse_update_clone_preserves_fields() {
        let update = DiffuseUpdate {
            step: 5,
            total_steps: 32,
            current_text: "Hello world".into(),
        };
        let cloned = update.clone();
        assert_eq!(cloned.step, 5);
        assert_eq!(cloned.total_steps, 32);
        assert_eq!(cloned.current_text, "Hello world");
    }

    #[test]
    fn test_diffuse_cmd_infill_carries_all_fields() {
        let (reply_tx, _reply_rx) = oneshot::channel();
        let cmd = DiffuseCmd::Infill {
            prefix: "The quick".into(),
            suffix: "jumped over".into(),
            mask_count: 20,
            steps_per_block: 4,
            block_length: 4,
            temperature: 0.7,
            dynamic_threshold: 0.9,
            reply: reply_tx,
        };

        // Verify the command can be pattern-matched (type-level test).
        match cmd {
            DiffuseCmd::Infill {
                mask_count,
                steps_per_block,
                block_length,
                ..
            } => {
                assert_eq!(mask_count, 20);
                assert_eq!(steps_per_block, 4);
                assert_eq!(block_length, 4);
            }
            _ => panic!("expected Infill variant"),
        }
    }
}
