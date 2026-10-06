"""Check read-only receipt guards and the maintained native-flow extension."""
import importlib.util
import json
from pathlib import Path
import sqlite3
import sys
import tempfile
from types import ModuleType
import unittest
from unittest.mock import patch

repository = ModuleType('gi.repository')
repository.Atspi = ModuleType('Atspi')
repository.GLib = ModuleType('GLib')
spec = importlib.util.spec_from_file_location(
    'recovery_capture', Path(__file__).with_name('qualify-child-plan-recovery.py'))
driver = importlib.util.module_from_spec(spec)
with patch.dict(sys.modules, {'pyatspi': ModuleType('pyatspi'), 'gi': ModuleType('gi'),
                             'gi.repository': repository}):
    spec.loader.exec_module(driver)


class RecoveryReceiptTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.database = Path(self.directory.name) / 'story.sqlite'
        self.connection = sqlite3.connect(self.database)
        self.addCleanup(self.connection.close)
        self.connection.executescript('''
            CREATE TABLE commands(id TEXT PRIMARY KEY, payload_type TEXT, payload_json TEXT);
            CREATE TABLE change_events(id TEXT PRIMARY KEY, command_id TEXT);
            CREATE TABLE child_plans(id TEXT PRIMARY KEY, parent_node_id TEXT, status TEXT, created_event_id TEXT);
            CREATE TABLE script_blocks(id TEXT PRIMARY KEY, text TEXT, revision TEXT);
            CREATE TABLE crdt(id TEXT PRIMARY KEY, state BLOB);
        ''')
        self.inputs = [{'block_id': 'a', 'text': driver.memory.MIDNIGHT,
                        'revision_event_id': 'saved-a', 'segment_revision_event_id': 'placement-a'},
                       {'block_id': 'b', 'text': driver.ui.PROPOSED_TEXT,
                        'revision_event_id': 'saved-b', 'segment_revision_event_id': 'placement-b'}]
        self.connection.execute('INSERT INTO commands VALUES(?,?,?)',
            ('create', 'ai.child_plan_create', json.dumps({'memory': {'script_inputs': self.inputs}})))
        self.connection.execute('INSERT INTO change_events VALUES(?,?)', ('created', 'create'))
        self.connection.execute('INSERT INTO child_plans VALUES(?,?,?,?)', ('plan', 'b', 'pending', 'created'))
        self.connection.execute('INSERT INTO script_blocks VALUES(?,?,?)', ('a', driver.memory.MIDNIGHT, 'saved-a'))
        self.connection.execute('INSERT INTO crdt VALUES(?,?)', ('a', b'\x00\xff\x81'))
        self.connection.commit()

    def test_reads_original_evidence_and_whole_database_without_writes(self):
        original_bytes = self.database.read_bytes()
        self.assertEqual(driver.stored_screenplay_inputs(self.database, 'plan'), self.inputs)
        before = driver.database_snapshot(self.database)
        self.assertEqual(before['table_row_counts']['crdt'], 1)
        driver.require_read_only_recovery(before, driver.database_snapshot(self.database), [], [])
        self.assertEqual(self.database.read_bytes(), original_bytes)

    def test_recovery_receipt_detects_text_history_status_and_source_receipt_changes(self):
        mutations = [
            "UPDATE script_blocks SET text='replacement'",
            "INSERT INTO change_events VALUES('extra','create')",
            "UPDATE child_plans SET status='applied'",
            "UPDATE commands SET payload_json='{}'",
        ]
        before = driver.database_snapshot(self.database)
        for sql in mutations:
            with self.subTest(sql=sql):
                self.connection.execute(sql)
                self.connection.commit()
                after = driver.database_snapshot(self.database)
                with self.assertRaisesRegex(RuntimeError, 'SQLite canonical state or history'):
                    driver.require_read_only_recovery(before, after, [], [])
                # Restore by using the original baseline again for the next
                # independently observed mutation; no UI write is being tested.
                before = after

    def test_provider_guard_counts_rejected_requests_too(self):
        snapshot = driver.database_snapshot(self.database)
        requests = [{'kind': 'child_initial', 'accepted': True, 'real_model': False}]
        with self.assertRaisesRegex(RuntimeError, 'extra provider request'):
            driver.require_read_only_recovery(snapshot, snapshot, requests,
                requests + [{'kind': 'child_initial', 'accepted': False, 'real_model': False}])

    def test_missing_legacy_evidence_does_not_claim_exact_recovery(self):
        self.connection.execute("UPDATE commands SET payload_json='{}'")
        self.connection.commit()
        with self.assertRaisesRegex(RuntimeError, 'no recorded canonical screenplay evidence'):
            driver.stored_screenplay_inputs(self.database, 'plan')
        with self.assertRaisesRegex(RuntimeError, 'exactly one original'):
            driver.stored_screenplay_inputs(self.database, 'missing')


class MaintainedDriverExtensionTests(unittest.TestCase):
    def test_recovery_runs_at_pending_capture_before_existing_stale_and_accept_flow(self):
        calls = []
        evidence = {'child_planning': {'stale_acceptance_refused_and_pending_retained': True,
            'explicit_timeline_acceptance_preserved_screenplay_and_bible': True}}

        def existing(*args):
            capture = args[-1]
            capture('eidetic-child-plan-pending.png')
            calls.append('existing manual edit and stale refusal')
            capture('eidetic-child-plan-refused.png')
            capture('eidetic-story-memory-readable.png')

        def recovery(*args):
            calls.append('recover through native UI')
            args[4]['child_plan_recovery'] = {}

        with patch.object(driver.memory, 'child_flow', existing), patch.object(
                driver, 'recover_pending_plan', recovery):
            driver.child_flow(None, None, None, None, evidence, None, calls.append)
        self.assertEqual(calls, ['eidetic-child-plan-pending.png', 'recover through native UI',
            'existing manual edit and stale refusal', 'eidetic-child-plan-refused.png',
            'eidetic-story-memory-readable.png'])
        self.assertTrue(evidence['child_plan_recovery']['later_manual_edit_refused_recovered_plan'])

    def test_changed_maintained_driver_cannot_silently_skip_recovery(self):
        with patch.object(driver.memory, 'child_flow'):
            with self.assertRaisesRegex(RuntimeError, 'skipped its required recovery point'):
                driver.child_flow(None, None, None, None, {}, None, lambda name: None)

    def test_duplicate_pending_checkpoint_fails_instead_of_repeating_native_input(self):
        def existing(*args):
            args[-1]('eidetic-child-plan-pending.png')
            args[-1]('eidetic-child-plan-pending.png')
        with patch.object(driver.memory, 'child_flow', existing), patch.object(
                driver, 'recover_pending_plan') as recovery:
            with self.assertRaisesRegex(RuntimeError, 'pending recovery point twice'):
                driver.child_flow(None, None, None, None, {}, None, lambda name: None)
            recovery.assert_called_once()


class NativeRefusalVisibilityTests(unittest.TestCase):
    def test_exact_hidden_native_refusal_is_revealed_after_expanded_evidence_scroll(self):
        # Exercise the actual maintained locator's native component scroll,
        # rather than declaring a hidden accessibility node already visible.
        message = 'Child plan story context changed; generate and review a fresh plan before accepting'
        showing, anywhere = object(), object()

        class NativeNode:
            def __init__(self, text, visible=False, children=()):
                self.name = text
                self.visible = visible
                self.children = children
                self.scroll_calls = []
            def __iter__(self):
                return iter(self.children)
            def clear_cache(self):
                pass
            def queryText(self):
                raise NotImplementedError
            def getState(self):
                node = self
                class State:
                    def contains(self, state):
                        return state is showing and node.visible
                return State()
            def queryComponent(self):
                return self
            def scrollTo(self, destination):
                self.scroll_calls.append(destination)
                self.visible = True

        lookalike = NativeNode('Child plan story context changed')
        refusal = NativeNode(message)
        application = NativeNode('Eidetic', visible=True, children=(lookalike, refusal))
        exact = lambda node: driver.ui.text_of(node).strip() == message
        with patch.object(driver.ui.pyatspi, 'STATE_SHOWING', showing, create=True), patch.object(
                driver.ui.pyatspi, 'SCROLL_ANYWHERE', anywhere, create=True):
            self.assertIsNone(driver.ui.find(application, exact))
            self.assertEqual(refusal.scroll_calls, [])
            self.assertIs(driver.ui.reveal(application, exact), refusal)
            self.assertTrue(driver.ui.visible(refusal))
        self.assertEqual(refusal.scroll_calls, [anywhere])
        self.assertEqual(lookalike.scroll_calls, [])


if __name__ == '__main__':
    unittest.main()
