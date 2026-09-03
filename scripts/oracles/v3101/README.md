<!-- Atipicial Chain · sovereign Layer-1 for smart contracts and digital assets -->
<!-- 👑 Founded & engineered by xmoohad — Blockchain Scientist · Computer Programmer -->

# Atipicial N3

These generators record observable results from the immutable upstream
revisions used by the Rust differential tests:

- Atipicial.VM: `004cd6070a940405818d9357638277dd44407e2e`
- Atipicial: `d10e9ceecdabe3fcff719ee68ea5b76ba7e62c3d`

The checked-in fixtures add only declarative inputs and hardfork applicability
to the generated records. `verify-recorded.py` compares every recorded
`observed` object to a fresh generator run and rejects missing or extra cases.

Run from the repository root after checking out the revisions above:

```bash
dotnet run --project scripts/oracles/v3101/atipicial-vm/atipicial-vm-oracle.csproj \
  --configuration Release -p:AtipicialVmSource=/path/to/atipicial-vm \
  > /tmp/atipicial-vm-v3101-oracle.json

dotnet run --project scripts/oracles/v3101/atipicial-application/atipicial-application-oracle.csproj \
  --configuration Release -p:AtipicialSource=/path/to/atipicial \
  > /tmp/atipicial-application-v3101-oracle.json

python3 scripts/oracles/v3101/verify-recorded.py \
  atipicial-vm/tests/fixtures/csharp-v3.10.1-vm.json \
  /tmp/atipicial-vm-v3101-oracle.json

python3 scripts/oracles/v3101/verify-recorded.py \
  atipicial-execution/tests/fixtures/csharp-v3.10.1-application.json \
  /tmp/atipicial-application-v3101-oracle.json
```

Do not update the recorded results from a moving branch or an unreviewed
revision. Rust production code never executes these C# projects.

---

> **Atipicial Chain** — sovereign Layer-1 for smart contracts and digital assets.
> 👑 Founded & engineered by **xmoohad** — Blockchain Scientist · Computer Programmer.
> `ATC` Atipicial Coin · `ATD` AtipicialDollar · addresses begin with **A**
