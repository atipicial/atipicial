import subprocess
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


class MakefileEntrypointTests(unittest.TestCase):
    def run_make_dry(self, target):
        return subprocess.run(
            ["make", "-n", target],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )

    def test_compose_exposes_service_metrics_ports(self):
        compose = (REPO_ROOT / "docker-compose.yml").read_text(encoding="utf-8")
        self.assertIn("${ATC_MAINNET_METRICS_PORT:-19090}:9090", compose)
        self.assertIn("${ATC_TESTNET_METRICS_PORT:-19091}:9091", compose)

    def test_compose_passes_observability_provider_tokens(self):
        compose = (REPO_ROOT / "docker-compose.yml").read_text(encoding="utf-8")
        self.assertIn("BETTER_STACK_SOURCE_TOKEN=${BETTER_STACK_SOURCE_TOKEN:-}", compose)
        self.assertIn(
            "GOOGLE_ERROR_REPORTING_TOKEN=${GOOGLE_ERROR_REPORTING_TOKEN:-}",
            compose,
        )
        self.assertIn("SENTRY_AUTH_HEADER=${SENTRY_AUTH_HEADER:-}", compose)

    def test_run_testnet_uses_atipicial_node_and_shipped_testnet_config(self):
        result = self.run_make_dry("run-testnet")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(
            "./target/debug/atipicial-node --config config/testnet.toml",
            result.stdout,
        )

    def test_run_service_testnet_uses_service_provider_preset(self):
        result = self.run_make_dry("run-service-testnet")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(
            "./target/debug/atipicial-node --config config/testnet-service.toml",
            result.stdout,
        )

    def test_compose_service_sets_service_profile(self):
        result = self.run_make_dry("compose-service")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(
            "ATC_PROFILE=service docker compose up -d atipicial-node",
            result.stdout,
        )


if __name__ == "__main__":
    unittest.main()
