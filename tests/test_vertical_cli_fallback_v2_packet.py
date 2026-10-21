from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class FVCv2PacketTest(unittest.TestCase):
    def test_complete_options_replay_every_fixed_command_and_score_without_new_inference(self):
        from verify_vertical_cli_fallback_v2 import verify_options
        result=verify_options(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2')
        self.assertEqual(result['CLI_processes'],384)
        self.assertEqual(result['verified_CSV_rows'],1152)
        self.assertEqual(result['comparisons'],576)
        self.assertEqual(result['regressions'],0)
        self.assertEqual(result['reject_all_empty_processes'],64)
        self.assertTrue(result['option_gate_pass'])
        self.assertFalse(result['new_accuracy_gain'])
        self.assertEqual(result['new_model_calls_in_replay'],0)

    def test_new_controls_sources_and_both_fresh_build_receipts_authenticate_before_recognition(self):
        from verify_vertical_cli_fallback_v2 import verify_before
        result=verify_before(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2')
        self.assertEqual(result['new_inputs'],72)
        self.assertEqual(result['known_inputs'],1239)
        self.assertEqual(result['annotated_glyphs'],4392)
        self.assertEqual(result['hash_overlap'],0)
        self.assertEqual(result['NN_calls_at_freeze'],0)

    def test_known_finish_regression_keeps_failed_attempts_and_real_quality_effect(self):
        from verify_vertical_cli_fallback_v2 import verify_known
        result=verify_known(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2')
        self.assertEqual(result['successful_settings'],3)
        self.assertEqual(result['all_recorded_settings'],9)
        self.assertEqual(result['independent_crop_RGBs'],9)
        self.assertEqual(result['preserved_failed_attempts'],3)
        self.assertTrue(result['quality_changes_initial'])
        self.assertFalse(result['new_accuracy_gain'])

    def test_feature_disabled_preserves_all_exits_stdout_errors_and_body_without_models(self):
        from verify_vertical_cli_fallback_v2 import verify_disabled
        result=verify_disabled(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2')
        self.assertEqual(result['CLI_processes'],16)
        self.assertEqual(result['compared_cases'],8)
        self.assertTrue(result['identical_exit_stdout_stderr_body'])
        self.assertEqual(result['NN_calls'],0)

    def test_complete_new72_replays_all_scores_and_excludes_the_stopped_attempt(self):
        from verify_vertical_cli_fallback_v2 import verify_new72
        result=verify_new72(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2')
        self.assertEqual(result['CLI_processes'],576)
        self.assertEqual(result['verified_CSV_rows'],1728)
        self.assertEqual(result['raw_gain_pairs'],22)
        self.assertEqual(result['unique_body_images_improved'],10)
        self.assertEqual(result['regressions'],0)
        self.assertEqual(result['accepted_stopped_results'],0)
        self.assertTrue(result['matrix_gate_pass'])

    def test_native_option_parser_preflight_keeps_its_failed_validator_and_runs_no_models(self):
        from verify_vertical_cli_fallback_v2 import verify_parser_preflight
        result=verify_parser_preflight(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2')
        self.assertEqual(result['parsed_argv'],384)
        self.assertEqual(result['successful_native_vectors'],48)
        self.assertEqual(result['preserved_failed_wrapper'],1)
        self.assertEqual(result['NN_calls'],0)
