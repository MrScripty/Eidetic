import importlib.util
from pathlib import Path
import sys
from types import ModuleType
import unittest
from unittest.mock import patch

repository=ModuleType('gi.repository')
repository.Atspi=ModuleType('Atspi')
repository.GLib=ModuleType('GLib')
spec=importlib.util.spec_from_file_location('recall_lifecycle',Path(__file__).with_name('qualify-bible-recall-lifecycle.py'))
driver=importlib.util.module_from_spec(spec)
with patch.dict(sys.modules,{'pyatspi':ModuleType('pyatspi'),'gi':ModuleType('gi'),'gi.repository':repository}):
    spec.loader.exec_module(driver)


class LifecycleCaptureTests(unittest.TestCase):
    def clean(self):
        return {'productionState':{'anchor':'qualification.mara','pending':False,'invalidated':False,'error':None,'projectionVersion':None}}

    def test_clean_remount_requires_exact_anchor_no_evidence_no_pending_no_error_no_false_mutation(self):
        value=self.clean()
        self.assertTrue(driver.clean(value))
        for key,bad in [('anchor',None),('pending',True),('invalidated',True),('error','late error'),('projectionVersion',1)]:
            with self.subTest(key=key):
                altered=self.clean()
                altered['productionState'][key]=bad
                self.assertFalse(driver.clean(altered))

    def test_observation_refuses_a_bad_client_frame_instead_of_claiming_one_good_frame(self):
        with patch.object(driver,'receipt',return_value=self.clean()),patch.object(driver.time,'monotonic',side_effect=[0,0.1,0.8]),patch.object(driver.time,'sleep'):
            seen=driver.observe(None,'clean',driver.clean)
            self.assertEqual(len(seen['samples']),1)
        bad=self.clean()
        bad['productionState']['pending']=True
        with patch.object(driver,'receipt',return_value=bad),patch.object(driver.time,'monotonic',side_effect=[0,0.1]):
            with self.assertRaisesRegex(RuntimeError,'invariant failed'):
                driver.observe(None,'clean',driver.clean)

    def test_pending_proposal_changes_are_detected_even_when_saved_screenplay_matches(self):
        with patch.object(driver.driver,'require_preserved'),patch.object(driver.ui,'wait_for'),patch.object(driver.ui,'query',return_value=[['changed','accepted']]):
            with self.assertRaisesRegex(RuntimeError,'pending proposal'):
                driver.preserved(None,None,{},[['original','pending']])


if __name__=='__main__':unittest.main()
