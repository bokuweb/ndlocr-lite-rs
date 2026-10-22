from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
from cli_cascade_trace import trace_command
from vertical_page_matrix import cli_command


class TraceCommandTest(unittest.TestCase):
    def setUp(self):
        self.assets={k:{'name':v} for k,v in {'deim':'det.onnx','100':'large.onnx','30':'small.onnx','50':'medium.onnx','charset':'chars.yaml'}.items()}
        self.binary=Path('/bin/frozen-cli');self.image=Path('/inputs/a b.png');self.models=Path('/weights');self.body=Path('/out/body.txt');self.missing=Path('/out/not-provided.onnx')

    def command(self, config):
        return trace_command(self.binary,self.image,self.models,self.assets,config,self.body,self.missing)

    def test_default_is_exact_existing_deim_command_with_no_setting_changes(self):
        self.assertEqual(self.command('default'),cli_command(self.binary,self.image,self.models,self.assets,'deim',self.body))

    def test_single_model_override_uses_existing_interface_without_optional_cascade_models(self):
        for config,name in [('model30','small.onnx'),('model50','medium.onnx')]:
            with self.subTest(config=config):
                argv=self.command(config)
                self.assertEqual(argv[argv.index('--model')+1],str(self.models/name))
                self.assertEqual(argv[argv.index('--model30')+1],str(self.missing))
                self.assertEqual(argv[argv.index('--model50')+1],str(self.missing))
                self.assertEqual(argv[argv.index('--enable-cascade')+1],'false')
                self.assertEqual(argv[argv.index('--image')+1],str(self.image))
                self.assertEqual(argv[argv.index('--det-model')+1],str(self.models/'det.onnx'))
                self.assertEqual(argv[argv.index('--charset')+1],str(self.models/'chars.yaml'))

    def test_unknown_config_is_not_silently_evaluated_as_a_different_model(self):
        with self.assertRaises(ValueError):self.command('unknown')

class TraceChangeGuardTest(unittest.TestCase):
    def test_only_changes_to_frozen_trace_or_runner_trigger_recognition(self):
        from cli_cascade_trace import trace_inputs_changed
        self.assertTrue(trace_inputs_changed(['tools/cli_cascade_trace.py']))
        self.assertTrue(trace_inputs_changed(['docs/benchmarks/vertical-cli-cascade-trace/sources.zip']))
        self.assertFalse(trace_inputs_changed(['docs/plan.md','src/infer/vertical_fallback.rs']))
        self.assertFalse(trace_inputs_changed(['docs/benchmarks/vertical-cli-cascade-trace/README.md',
                                                    'docs/benchmarks/vertical-cli-cascade-trace/results-v1.zip']))



class TraceFrozenFixtureTest(unittest.TestCase):
    def test_existing_compile_time_fixture_is_bound_in_both_frozen_sources(self):
        from vertical_page_matrix import Bundle,sha
        bundle=Bundle(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-cascade-trace')
        for variant in ['baseline','candidate']:
            self.assertEqual(sha(bundle.read('sources.zip',variant+'/tests/fixtures/line_segment/jma-notice-body.png')),
                             'c906f07dbecade29a0484032581a9e6690c3bc6a5c62fead73c57e7a669fc799')

if __name__=='__main__':unittest.main()
