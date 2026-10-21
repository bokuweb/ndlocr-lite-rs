from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))


class FVCv2OptionsTest(unittest.TestCase):
    def inputs(self):
        import json
        from verify_additional_real_baseline import _packet
        _,data=_packet(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2','before-nn-v1.index.json')
        return json.loads(data['evidence/new-manifest.json'])['inputs']

    def test_all_profiles_routes_variants_and_fixed_inputs_are_planned_before_nn(self):
        from vertical_cli_fallback_v2_options import option_plan
        rows=option_plan(self.inputs())
        self.assertEqual(len(rows),384)
        self.assertEqual(len({(r['id'],r['profile'],r['deim'],r['cascade'],r['variant']) for r in rows}),384)
        self.assertEqual({r['variant'] for r in rows},{'baseline','candidate'})
        self.assertEqual({r['profile'] for r in rows},{'padding5','reject_all','threshold180','threshold240','quality','custom'})
        self.assertEqual(len({r['id'] for r in rows}),8)
        self.assertEqual(sum(r['profile']=='reject_all' for r in rows),64)

    def test_partial_duplicate_or_unknown_input_sets_do_not_silently_change_the_scope(self):
        from vertical_cli_fallback_v2_options import option_plan
        inputs=self.inputs()
        for rows in [inputs[:-1],inputs+inputs[:1],[dict(r,id='other') if i==0 else r for i,r in enumerate(inputs)]]:
            with self.assertRaises(ValueError):option_plan(rows)

    def test_every_setting_is_explicit_and_gt_is_not_passed_to_cli(self):
        from vertical_cli_fallback_v2_options import option_plan,option_command
        assets={key:{'name':key+'.fixed'} for key in ['deim','100','50','30','charset']}
        for row in option_plan(self.inputs()):
            self.assertNotIn('truth',row)
            argv=option_command(row,'/frozen/'+row['variant'],Path('/models'),assets,Path('/body.txt'))
            self.assertEqual(argv[0],'/frozen/'+row['variant'])
            self.assertEqual(argv[argv.index('--image')+1],row['image'])
            self.assertEqual(argv[argv.index('--use-deim-detection')+1],str(row['deim']).lower())
            self.assertEqual(argv[argv.index('--enable-cascade')+1],str(row['cascade']).lower())
            for name,value in row['options'].items():
                option='--'+name.replace('_','-')
                self.assertEqual(argv.count(option),1)
                self.assertEqual(argv[argv.index(option)+1],str(value).lower())
            self.assertEqual(argv[argv.index('--output-txt')+1],'/body.txt')

    def test_public_option_plan_replays_all_frozen_argv_without_running_them(self):
        from verify_vertical_cli_fallback_v2 import verify_option_plan
        result=verify_option_plan(Path(__file__).resolve().parents[1]/'docs/benchmarks/vertical-cli-fallback-v2')
        self.assertEqual(result['planned_commands'],384)
        self.assertEqual(result['planned_CSV_rows'],1152)
        self.assertEqual(result['option_matrix_NN_calls'],0)
