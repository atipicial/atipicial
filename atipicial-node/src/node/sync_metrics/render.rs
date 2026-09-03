//! Prometheus text rendering for node sync metrics.

use super::families::{
    append_mdbx_commit_metrics, append_native_contract_hooks, append_native_persist_tx_stages,
    append_atipicial_coin_candidate_counts, append_atipicial_coin_committee_compute_stages,
    append_atipicial_coin_onpersist_stages, append_state_root_apply_metrics,
};

/// Render sync metrics as Prometheus-format text.
pub fn render_prometheus() -> String {
    use atipicial_runtime::sync_metrics as m;

    let height = m::height();
    let peer_tip = m::peer_live_tip();
    let blocks = m::blocks_persisted();
    let state_apply = atipicial_state_service::StateRootApplyMetrics::state_root_apply_stats();

    let mut output = format!(
        "# HELP atipicial_sync_height Current block height\n\
         # TYPE atipicial_sync_height gauge\n\
         atipicial_sync_height {height}\n\
         # HELP atipicial_sync_peer_tip Peer-reported live chain tip\n\
         # TYPE atipicial_sync_peer_tip gauge\n\
         atipicial_sync_peer_tip {peer_tip}\n\
         # HELP atipicial_sync_lag Blocks behind live tip\n\
         # TYPE atipicial_sync_lag gauge\n\
         atipicial_sync_lag {}\n\
         # HELP atipicial_sync_blocks_persisted Total blocks persisted since startup\n\
         # TYPE atipicial_sync_blocks_persisted counter\n\
         atipicial_sync_blocks_persisted {blocks}\n\
         # HELP atipicial_sync_headers_downloaded_total Headers received from correlated P2P range responses\n\
         # TYPE atipicial_sync_headers_downloaded_total counter\n\
         atipicial_sync_headers_downloaded_total {}\n\
         # HELP atipicial_sync_headers_verified_total Headers durably accepted by the headers stage\n\
         # TYPE atipicial_sync_headers_verified_total counter\n\
         atipicial_sync_headers_verified_total {}\n\
         # HELP atipicial_sync_headers_checkpoint_height Durable headers-stage checkpoint height\n\
         # TYPE atipicial_sync_headers_checkpoint_height gauge\n\
         atipicial_sync_headers_checkpoint_height {}\n\
         # HELP atipicial_sync_header_fetch_failures_total Failed or rejected correlated header fetches\n\
         # TYPE atipicial_sync_header_fetch_failures_total counter\n\
         atipicial_sync_header_fetch_failures_total {}\n\
         # HELP atipicial_sync_bodies_checkpoint_height Durable bodies-stage checkpoint height\n\
         # TYPE atipicial_sync_bodies_checkpoint_height gauge\n\
         atipicial_sync_bodies_checkpoint_height {}\n\
         # HELP atipicial_sync_body_header_mismatches_total Downloaded blocks rejected for disagreeing with verified headers\n\
         # TYPE atipicial_sync_body_header_mismatches_total counter\n\
         atipicial_sync_body_header_mismatches_total {}\n\
         # HELP atipicial_sync_avg_total_us EWMA total per-block persist time (microseconds)\n\
         # TYPE atipicial_sync_avg_total_us gauge\n\
         atipicial_sync_avg_total_us {}\n\
         # HELP atipicial_sync_avg_verify_us EWMA witness verification time (microseconds)\n\
         # TYPE atipicial_sync_avg_verify_us gauge\n\
         atipicial_sync_avg_verify_us {}\n\
         # HELP atipicial_sync_avg_persist_us EWMA native contract execution time (microseconds)\n\
         # TYPE atipicial_sync_avg_persist_us gauge\n\
         atipicial_sync_avg_persist_us {}\n\
         # HELP atipicial_sync_avg_commit_us EWMA persistent-store commit time (microseconds)\n\
         # TYPE atipicial_sync_avg_commit_us gauge\n\
         atipicial_sync_avg_commit_us {}\n\
         # HELP atipicial_sync_native_persist_blocks_total Total native persistence records since startup\n\
         # TYPE atipicial_sync_native_persist_blocks_total counter\n\
         atipicial_sync_native_persist_blocks_total {}\n\
         # HELP atipicial_sync_native_persist_height Latest block height observed by native persistence metrics\n\
         # TYPE atipicial_sync_native_persist_height gauge\n\
         atipicial_sync_native_persist_height {}\n\
         # HELP atipicial_sync_native_persist_avg_total_us EWMA total native persistence time (microseconds)\n\
         # TYPE atipicial_sync_native_persist_avg_total_us gauge\n\
         atipicial_sync_native_persist_avg_total_us {}\n\
         # HELP atipicial_sync_native_persist_avg_onpersist_us EWMA native OnPersist stage time (microseconds)\n\
         # TYPE atipicial_sync_native_persist_avg_onpersist_us gauge\n\
         atipicial_sync_native_persist_avg_onpersist_us {}\n\
         # HELP atipicial_sync_native_persist_avg_tx_us EWMA per-transaction Application stage time (microseconds)\n\
         # TYPE atipicial_sync_native_persist_avg_tx_us gauge\n\
         atipicial_sync_native_persist_avg_tx_us {}\n\
         # HELP atipicial_sync_native_persist_avg_postpersist_us EWMA native PostPersist stage time (microseconds)\n\
         # TYPE atipicial_sync_native_persist_avg_postpersist_us gauge\n\
         atipicial_sync_native_persist_avg_postpersist_us {}\n\
         # HELP atipicial_sync_native_persist_avg_cache_commit_us EWMA native persistence staged cache merge time (microseconds)\n\
         # TYPE atipicial_sync_native_persist_avg_cache_commit_us gauge\n\
         atipicial_sync_native_persist_avg_cache_commit_us {}\n\
         # HELP atipicial_sync_native_persist_avg_tx_count EWMA transaction count per native persistence call\n\
         # TYPE atipicial_sync_native_persist_avg_tx_count gauge\n\
         atipicial_sync_native_persist_avg_tx_count {}\n\
         # HELP atipicial_state_service_mpt_apply_blocks_total Total local StateService MPT apply attempts\n\
         # TYPE atipicial_state_service_mpt_apply_blocks_total counter\n\
         atipicial_state_service_mpt_apply_blocks_total {}\n\
         # HELP atipicial_state_service_mpt_apply_failures_total Total failed local StateService MPT apply attempts\n\
         # TYPE atipicial_state_service_mpt_apply_failures_total counter\n\
         atipicial_state_service_mpt_apply_failures_total {}\n\
         # HELP atipicial_state_service_mpt_apply_height Latest block height observed by local StateService MPT apply\n\
         # TYPE atipicial_state_service_mpt_apply_height gauge\n\
         atipicial_state_service_mpt_apply_height {}\n\
         # HELP atipicial_state_service_mpt_apply_avg_total_us EWMA total local StateService MPT apply time (microseconds)\n\
         # TYPE atipicial_state_service_mpt_apply_avg_total_us gauge\n\
         atipicial_state_service_mpt_apply_avg_total_us {}\n\
         # HELP atipicial_state_service_mpt_apply_avg_project_us EWMA DataCache-to-MPT changeset projection time (microseconds)\n\
         # TYPE atipicial_state_service_mpt_apply_avg_project_us gauge\n\
         atipicial_state_service_mpt_apply_avg_project_us {}\n\
         # HELP atipicial_state_service_mpt_apply_avg_trie_us EWMA trie/write local StateService MPT apply time (microseconds)\n\
         # TYPE atipicial_state_service_mpt_apply_avg_trie_us gauge\n\
         atipicial_state_service_mpt_apply_avg_trie_us {}\n\
         # HELP atipicial_state_service_mpt_apply_avg_changes EWMA projected StateService MPT changes per block\n\
         # TYPE atipicial_state_service_mpt_apply_avg_changes gauge\n\
         atipicial_state_service_mpt_apply_avg_changes {}\n",
        peer_tip.saturating_sub(height),
        m::headers_downloaded(),
        m::headers_verified(),
        m::headers_checkpoint_height(),
        m::header_fetch_failures(),
        m::bodies_checkpoint_height(),
        m::body_header_mismatches(),
        m::avg_total_us(),
        m::avg_verify_us(),
        m::avg_persist_us(),
        m::avg_commit_us(),
        m::native_persist_blocks(),
        m::native_persist_height(),
        m::native_persist_avg_total_us(),
        m::native_persist_avg_onpersist_us(),
        m::native_persist_avg_tx_us(),
        m::native_persist_avg_postpersist_us(),
        m::native_persist_avg_cache_commit_us(),
        m::native_persist_avg_tx_count(),
        state_apply.attempts,
        state_apply.failures,
        state_apply.latest_height,
        state_apply.avg_total_us,
        state_apply.avg_project_us,
        state_apply.avg_apply_us,
        state_apply.avg_changes,
    );

    append_state_root_apply_metrics(&mut output);
    append_mdbx_commit_metrics(&mut output);
    append_native_contract_hooks(&mut output);
    append_native_persist_tx_stages(&mut output);
    append_atipicial_coin_onpersist_stages(&mut output);
    append_atipicial_coin_committee_compute_stages(&mut output);
    append_atipicial_coin_candidate_counts(&mut output);

    output
}

#[cfg(test)]
mod tests {
    use super::render_prometheus;

    #[test]
    fn render_prometheus_keeps_labelled_metric_families() {
        atipicial_state_service::metrics::StateRootApplyMetrics::record_stage(
            atipicial_state_service::metrics::StateRootApplyStage::RootHash,
            17,
        );
        atipicial_state_service::metrics::StateRootApplyMetrics::record_count(
            atipicial_state_service::metrics::StateRootApplyCountKind::OverlayEntries,
            3,
        );
        atipicial_runtime::sync_metrics::record_native_contract_hook(
            atipicial_runtime::sync_metrics::NativePersistHook::OnPersist,
            -5,
            23,
        );
        atipicial_runtime::sync_metrics::record_native_persist_tx_stage(
            atipicial_runtime::sync_metrics::NativePersistTxStage::Execute,
            29,
        );
        atipicial_runtime::sync_metrics::record_atipicial_coin_onpersist_stage(
            atipicial_runtime::sync_metrics::AtipicialCoinOnPersistStage::RefreshTotal,
            31,
        );
        atipicial_runtime::sync_metrics::record_atipicial_coin_committee_compute_stage(
            atipicial_runtime::sync_metrics::AtipicialCoinCommitteeComputeStage::TopCandidateMaintenance,
            37,
        );
        atipicial_runtime::sync_metrics::record_atipicial_coin_committee_candidate_count(
            atipicial_runtime::sync_metrics::AtipicialCoinCommitteeCandidateCount::EligibleCandidates,
            5,
        );

        let output = render_prometheus();

        assert!(output.contains("# HELP atipicial_sync_height Current block height"));
        assert!(output.contains("atipicial_sync_headers_downloaded_total "));
        assert!(output.contains("atipicial_sync_headers_verified_total "));
        assert!(output.contains("atipicial_sync_headers_checkpoint_height "));
        assert!(output.contains("atipicial_sync_header_fetch_failures_total "));
        assert!(output.contains("atipicial_sync_bodies_checkpoint_height "));
        assert!(output.contains("atipicial_sync_body_header_mismatches_total "));
        assert!(
            output.contains("atipicial_state_service_mpt_apply_stage_calls_total{stage=\"root_hash\"} "),
            "state-root stage labels should stay Prometheus-compatible"
        );
        assert!(
            output.contains(
                "atipicial_state_service_mpt_apply_count_samples_total{kind=\"overlay_entries\"} "
            ),
            "state-root count labels should stay Prometheus-compatible"
        );
        assert!(
            output.contains(
                "atipicial_sync_native_contract_hook_calls_total{trigger=\"onpersist\",contract=\"AtipicialCoin\",id=\"-5\"} "
            ),
            "native hook labels should preserve trigger, contract, and id"
        );
        assert!(
            output.contains("atipicial_sync_native_persist_tx_stage_calls_total{stage=\"execute\"} "),
            "native transaction-stage labels should stay stable"
        );
        assert!(
            output.contains("atipicial_sync_native_persist_tx_stage_total_us{stage=\"execute\"} "),
            "native transaction-stage cumulative timing should stay labelled"
        );
        assert!(
            output.contains(
                "atipicial_sync_atipicialtoken_onpersist_stage_calls_total{stage=\"refresh_total\"} "
            ),
            "AtipicialCoin OnPersist stage labels should stay stable"
        );
        assert!(
            output.contains(
                "atipicial_sync_atipicialtoken_committee_compute_stage_calls_total{stage=\"top_candidate_maintenance\"} "
            ),
            "AtipicialCoin committee-compute stage labels should stay stable"
        );
        assert!(
            output.contains(
                "atipicial_sync_atipicialtoken_committee_candidate_scan_samples_total{kind=\"eligible_candidates\"} "
            ),
            "AtipicialCoin candidate-scan count labels should stay stable"
        );
    }
}
