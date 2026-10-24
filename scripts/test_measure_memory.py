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
                self.assertIn('input', operations)


if __name__ == '__main__': unittest.main()
