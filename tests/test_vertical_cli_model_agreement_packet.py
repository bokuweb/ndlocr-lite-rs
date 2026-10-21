from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class CliAgreementPacketTest(unittest.TestCase):
    def test_before_nn_packet_binds_sources_fresh_builds_and_all_96_new_controls(self):
        from verify_vertical_cli_model_agreement import verify
        result=verify(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-model-agreement')
        self.assertEqual(result['controls'],96)
        self.assertEqual(result['glyphs'],4860)
        self.assertEqual(result['known_physical_inputs'],1143)
        self.assertEqual(result['duplicate_hashes'],0)
        self.assertEqual(result['fresh_own_artifacts'],6)
        self.assertFalse(result['production_hook'])
        self.assertEqual(result['NN_calls_at_packet_creation'],0)

class CliAgreementKnownRegressionTest(unittest.TestCase):
    def test_literal_assertion_proof_is_bound_to_the_same_frozen_sources(self):
        from verify_vertical_cli_model_agreement import verify_known
        result=verify_known(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-model-agreement')
        self.assertEqual(result['CLI_processes'],2)
        self.assertEqual(result['baseline_body_scalars'],24)
        self.assertEqual(result['candidate_body_scalars_verified_by_test'],71)
        self.assertEqual(result['new_control_gains'],0)
        self.assertFalse(result['candidate_raw_body_retained'])
