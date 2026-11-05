#!/usr/bin/env python3
"""Regression checks for preserving the existing memory measurement workloads."""
import unittest
import measure_memory


class MemoryWorkloadTests(unittest.TestCase):
    def test_ncd_controls_do_not_leak_into_existing_workloads(self):
        # Adding the paired NCD control previously changed existing benchmark
        # definitions, invalidating comparisons with their retained baselines.
        for name, workloads in [
            ('smoke', measure_memory.cases(True)),
            ('default', measure_memory.cases(False)),
            ('lz76', measure_memory.lz76_cases()),
            ('subword', measure_memory.subword_cases()),
        ]:
            with self.subTest(workload=name):
                operations = {case[0] for case in workloads}
                self.assertNotIn('ncd_input', operations)
                self.assertNotIn('ncd', operations)
                self.assertNotIn('windows', operations)
                self.assertNotIn('blocks', operations)
                self.assertNotIn('statistical_complexity', operations)
                self.assertNotIn('block_entropy', operations)
                self.assertNotIn('block_entropy_reuse', operations)
                self.assertNotIn('conditional_order', operations)
                self.assertNotIn('conditional_order_reuse', operations)
                self.assertTrue({'block_profile','block_profile_metadata','block_profile_select'}.isdisjoint(operations))
                self.assertTrue({'joint_input','joint_counts','joint_entropy','joint_reuse','joint_marginals'}.isdisjoint(operations))
                self.assertTrue({'mutual_input','mutual','mutual_reuse'}.isdisjoint(operations))
                self.assertIn('input', operations)


    def test_conditional_operations_do_not_change_block_entropy_workloads(self):
        operations = {case[0] for case in measure_memory.block_entropy_cases()}
        self.assertEqual(operations, {'input', 'block_entropy', 'block_entropy_reuse'})


    def test_profiles_do_not_change_conditional_workloads(self):
        operations = {case[0] for case in measure_memory.conditional_order_cases()}
        self.assertEqual(operations, {'input','conditional_order','conditional_order_reuse'})


    def test_joint_work_does_not_change_profile_workloads(self):
        operations = {case[0] for case in measure_memory.block_profile_cases()}
        self.assertEqual(operations, {'input','block_profile','block_profile_metadata','block_profile_select'})


    def test_mutual_work_does_not_change_joint_workloads(self):
        operations = {case[0] for case in measure_memory.joint_cases()}
        self.assertEqual(operations, {'joint_input','joint_counts','joint_entropy','joint_reuse','joint_marginals'})


    def test_lz76_normalization_preserves_mutual_workloads(self):
        self.assertEqual({case[0] for case in measure_memory.mutual_cases()}, {'mutual_input','mutual','mutual_reuse'})

    def test_normalized_lz76_bounds_diverse_workloads(self):
        cases = list(measure_memory.lz76_normalized_cases())
        self.assertEqual(len(cases), 90)
        self.assertTrue(all(size <= 65536 for _, shape, size, _ in cases if shape == 'mixed'))
        self.assertEqual({case[0] for case in cases}, {'input','lz76_density','lz76_asymptotic','lz76_summary','lz76_normalized_reuse'})

if __name__ == '__main__': unittest.main()
