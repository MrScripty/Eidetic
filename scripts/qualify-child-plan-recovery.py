#!/usr/bin/env python3
"""Recover an existing child proposal through the actual native application UI.

Extends the maintained Bible/child-plan walkthrough at its pending capture.
AT-SPI/X11 input only; SQLite observations are read-only. HTTP responses are
synthetic fixtures and do not qualify real-model quality.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sqlite3

spec = importlib.util.spec_from_file_location(
    'child_memory_capture', Path(__file__).with_name('qualify-child-plan-memory.py'))
memory = importlib.util.module_from_spec(spec)
spec.loader.exec_module(memory)
ui = memory.ui


def database_snapshot(database):
    """Hash all logical rows in one read transaction, including receipt/history."""
    with sqlite3.connect(database.as_uri() + '?mode=ro', uri=True, timeout=2) as connection:
        connection.execute('BEGIN')
        tables = [row[0] for row in connection.execute(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")]
        contents = {}
        for name in tables:
            identifier = '"' + name.replace('"', '""') + '"'
            rows = connection.execute('SELECT * FROM ' + identifier).fetchall()
            contents[name] = sorted(json.dumps(row, ensure_ascii=False, separators=(',', ':'),
                default=lambda value: {'bytes_hex': value.hex()}) for row in rows)
        encoded = json.dumps(contents, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode('utf-8')
        return {'sha256': hashlib.sha256(encoded).hexdigest(),
                'table_row_counts': {name: len(rows) for name, rows in contents.items()}}


def stored_screenplay_inputs(database, plan_id):
    rows = ui.query(database,
        "SELECT c.payload_json FROM child_plans p JOIN change_events e ON e.id=p.created_event_id "
        "JOIN commands c ON c.id=e.command_id WHERE p.id=? AND c.payload_type='ai.child_plan_create'",
        (plan_id,))
    if len(rows) != 1:
        raise RuntimeError('Expected exactly one original child plan creation receipt')
    command = json.loads(rows[0][0])
    receipt = command.get('memory')
    inputs = receipt.get('script_inputs') if isinstance(receipt, dict) else None
    if not isinstance(inputs, list):
        raise RuntimeError('Pending plan has no recorded canonical screenplay evidence')
    return inputs


def provider_records():
    with memory.ChildProvider.records_lock:
        return json.loads(json.dumps(memory.ChildProvider.records))


def require_read_only_recovery(before, after, requests_before, requests_after):
    if after != before:
        raise RuntimeError('Reading/reopening a saved child plan changed SQLite canonical state or history')
    if requests_after != requests_before:
        raise RuntimeError('Reading/reopening a saved child plan made an extra provider request')


def recover_pending_plan(application, window, database, fixture, evidence, checkpoint, capture):
    plan_id = evidence['child_planning']['initial_plan_id']
    parent = fixture['b']['id']
    pending = ui.query(database,
        "SELECT id FROM child_plans WHERE parent_node_id=? AND status='pending'", (parent,))
    if pending != [(plan_id,)]:
        raise RuntimeError('Native qualification requires one unambiguous pending target plan')
    inputs = stored_screenplay_inputs(database, plan_id)
    if not any(row['text'] == memory.MIDNIGHT for row in inputs) or not any(
            row['text'] == ui.PROPOSED_TEXT for row in inputs):
        raise RuntimeError('Original pending plan omitted exact midnight/source screenplay')
    before, requests_before = database_snapshot(database), provider_records()

    checkpoint('close pending child preview and navigate away without applying')
    ui.reveal_button(application, 'Close preview', window)

    def review_absent():
        application.clear_cache()
        return not any(n.name == 'Review proposed timeline children' for n in ui.walk(application))

    ui.wait_for('pending preview closed in native UI', review_absent)
    for key in ('a', 'b'):
        selected = ui.wait_for('native scene navigation ' + key,
            lambda key=key: ui.native_timeline_clip(application, fixture[key]['name'], window))
        ui.click_control(selected[0], window, selected[1])
        # Selecting A then B must settle before the next action. The existing
        # selected-node editor's heading names the actual selected clip.
        ui.wait_for('selected editor ' + fixture[key]['name'], lambda key=key: ui.reveal(
            application, lambda n: n.getRole() == ui.pyatspi.ROLE_HEADING
            and n.name == fixture[key]['name']))

    ui.reveal_button(application, 'Review saved timeline plans', window)
    section = ui.wait_for('saved timeline plans in real UI', lambda: ui.reveal(
        application, lambda n: n.name == 'Saved timeline plans'
        and any('Midnight departure' in ui.text_of(child) for child in ui.walk(n))))
    evidence['child_plan_recovery'] = {'plan_id': plan_id, 'parent_node_id': parent,
        'original_script_inputs': inputs, 'database_before': before,
        'provider_request_count_before': len(requests_before),
        'route': ['Close preview', 'select another scene', 'reselect target scene',
                  'Review saved timeline plans', 'Review plan 1'],
        'real_model': False}
    checkpoint('stored pending plan listed after native selection away and back')
    capture('eidetic-child-plan-recovery-list.png')
    ui.reveal_button(section, 'Review plan 1', window)

    def recovered_review():
        section = ui.reveal(application, lambda n: n.name == 'Review proposed timeline children')
        if section:
            texts = [ui.text_of(n) for n in ui.walk(section)]
            if any('Midnight departure' in text for text in texts) and any(
                    'Mara takes her blue umbrella to the midnight train.' in text for text in texts):
                return section
        return None

    review = ui.wait_for('exact stored canonical child proposal reopened', recovered_review)
    summary = ui.wait_for('saved evidence disclosure in recovered plan', lambda: ui.reveal(
        review, lambda n: n.name == 'Saved screenplay used for this plan'))
    ui.click_control(summary, window)
    for row in inputs:
        ui.wait_for('original screenplay evidence ' + row['block_id'], lambda row=row: ui.reveal(
            review, lambda n: ui.text_of(n) == row['text']))
    after, requests_after = database_snapshot(database), provider_records()
    require_read_only_recovery(before, after, requests_before, requests_after)
    evidence['child_plan_recovery'].update(database_after=after,
        provider_request_count_after=len(requests_after),
        exact_original_screenplay_evidence_visible=True,
        exact_canonical_child_proposal_reopened=True,
        canonical_state_history_and_receipt_unchanged=True,
        no_extra_provider_request=True,
        explicit_acceptance_still_required=True)
    checkpoint('recovered original proposal and source evidence; no writes or provider call')
    capture('eidetic-child-plan-recovered.png')


def child_flow(application, window, database, fixture, evidence, checkpoint, capture):
    recovered = False

    def capture_with_recovery(name):
        nonlocal recovered
        capture(name)
        if name != 'eidetic-child-plan-pending.png':
            return
        if recovered:
            raise RuntimeError('Maintained child driver reached its pending recovery point twice')
        recover_pending_plan(application, window, database, fixture, evidence, checkpoint, capture)
        recovered = True

    memory.child_flow(application, window, database, fixture, evidence, checkpoint, capture_with_recovery)
    if not recovered:
        raise RuntimeError('Maintained child driver skipped its required recovery point')
    evidence['child_plan_recovery'].update(
        status='native_saved_child_plan_recovery_passed_with_synthetic_http_fixture',
        later_manual_edit_refused_recovered_plan=evidence['child_planning'][
            'stale_acceptance_refused_and_pending_retained'],
        later_fresh_plan_explicitly_accepted=evidence['child_planning'][
            'explicit_timeline_acceptance_preserved_screenplay_and_bible'])


if __name__ == '__main__':
    if not os.environ.get('EIDETIC_CAPTURE_SOURCE'):
        raise SystemExit('An exact frozen EIDETIC_CAPTURE_SOURCE is required')
    memory.bible.BibleFactProvider = memory.ChildProvider
    memory.bible.AFTER_ACCEPTANCE = child_flow
    memory.bible.main()
