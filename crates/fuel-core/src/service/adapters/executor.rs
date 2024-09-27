use crate::{
    database::RelayerIterableKeyValueView,
    service::adapters::TransactionsSource,
};
use fuel_core_executor::ports::MaybeCheckedTransaction;
use fuel_core_types::{
    blockchain::primitives::DaBlockHeight,
    services::relayer::Event,
};

impl fuel_core_executor::ports::TransactionsSource for TransactionsSource {
    // TODO: Use `size_limit` https://github.com/FuelLabs/fuel-core/issues/2133
    fn next(
        &self,
        gas_limit: u64,
        transactions_limit: u16,
        _: u32,
    ) -> Vec<MaybeCheckedTransaction> {
        match self
            .txpool
            .select_transactions(gas_limit, transactions_limit)
        {
            Ok(txs) => txs
                .into_iter()
                .map(|tx| {
                    MaybeCheckedTransaction::CheckedTransaction(
                        tx.as_ref().into(),
                        tx.used_consensus_parameters_version(),
                    )
                })
                .collect(),
            Err(e) => {
                tracing::warn!(
                    "Error when trying to get the transactions {}",
                    e.to_string()
                );
                Vec::new()
            }
        }
    }
}

impl fuel_core_executor::ports::RelayerPort for RelayerIterableKeyValueView {
    fn enabled(&self) -> bool {
        #[cfg(feature = "relayer")]
        {
            true
        }
        #[cfg(not(feature = "relayer"))]
        {
            false
        }
    }

    fn get_events(&self, da_height: &DaBlockHeight) -> anyhow::Result<Vec<Event>> {
        #[cfg(feature = "relayer")]
        {
            use fuel_core_storage::StorageAsRef;
            let events = self
                .storage::<fuel_core_relayer::storage::EventsHistory>()
                .get(da_height)?
                .map(|cow| cow.into_owned())
                .unwrap_or_default();
            Ok(events)
        }
        #[cfg(not(feature = "relayer"))]
        {
            let _ = da_height;
            Ok(vec![])
        }
    }
}
