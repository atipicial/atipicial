import unittest

from scripts.tests.file_size_policy import (
    REPO_ROOT,
    RUST_SIZE_BASELINE,
    assert_line_budget,
    baseline_for_paths,
    rust_source_files,
)


CONSENSUS_CRITICAL_CRATES = (
    "atipicial-blockchain",
    "atipicial-config",
    "atipicial-consensus",
    "atipicial-crypto",
    "atipicial-execution",
    "atipicial-mempool",
    "atipicial-native-contracts",
    "atipicial-payloads",
    "atipicial-primitives",
    "atipicial-state-service",
    "atipicial-storage",
    "atipicial-vm",
)


class CoreFileSizeLimitTests(unittest.TestCase):
    def test_consensus_critical_sources_obey_the_repository_size_policy(self):
        paths = [
            path
            for crate in CONSENSUS_CRITICAL_CRATES
            for path in rust_source_files(REPO_ROOT / crate)
        ]
        assert_line_budget(
            self,
            paths,
            baseline_for_paths(paths, RUST_SIZE_BASELINE),
            minimum_files=300,
        )


if __name__ == "__main__":
    unittest.main()
