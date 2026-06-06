use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    vec, Address, Env, Symbol,
};

use crate::types::CategorySpend;
use crate::{CategoryAnalytics, CategoryAnalyticsClient};
// handle hash
fn setup_analytics_contract() -> (Env, Address, CategoryAnalyticsClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CategoryAnalytics, ());
    let client = CategoryAnalyticsClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init(&admin);

    (env, admin, client)
}

#[test]
fn test_integration_spending_trends() {
    let (env, _admin, client) = setup_analytics_contract();
    let user = Address::generate(&env);
    let category = Symbol::new(&env, "shopping");

    // Year/Month calculation in contract:
    // 2026: 1768608000
    env.ledger().set_timestamp(1768608000);

    client.record_spending(&user, &category, &5000);
    client.record_spending(&user, &category, &3000);

    let metrics = client.get_category_metrics(&user, &category, &2026, &2);
    assert_eq!(metrics.volume, 8000);
    assert_eq!(metrics.count, 2);

    // Advance time to March (approx 30 days)
    env.ledger().set_timestamp(1768608000 + 2592000);
    client.record_spending(&user, &category, &2000);

    let march_metrics = client.get_category_metrics(&user, &category, &2026, &3);
    assert_eq!(march_metrics.volume, 2000);

    let yearly = client.get_yearly_trend(&user, &category, &2026);
    assert_eq!(yearly.volume, 10000);
    assert_eq!(yearly.count, 3);
}

#[test]
fn test_multi_category_batch_aggregation() {
    let (env, _admin, client) = setup_analytics_contract();
    let user = Address::generate(&env);
    let shopping = Symbol::new(&env, "shopping");
    let groceries = Symbol::new(&env, "groceries");
    let transport = Symbol::new(&env, "transport");

    env.ledger().set_timestamp(1768608000);

    let spendings = vec![
        &env,
        CategorySpend {
            category: shopping.clone(),
            amount: 5000,
        },
        CategorySpend {
            category: groceries.clone(),
            amount: 2500,
        },
        CategorySpend {
            category: shopping.clone(),
            amount: 1500,
        },
        CategorySpend {
            category: transport.clone(),
            amount: 900,
        },
    ];

    client.record_spending_batch(&user, &spendings);

    let shopping_metrics = client.get_category_metrics(&user, &shopping, &2026, &2);
    assert_eq!(shopping_metrics.volume, 6500);
    assert_eq!(shopping_metrics.count, 2);

    let groceries_metrics = client.get_category_metrics(&user, &groceries, &2026, &2);
    assert_eq!(groceries_metrics.volume, 2500);
    assert_eq!(groceries_metrics.count, 1);

    let transport_metrics = client.get_category_metrics(&user, &transport, &2026, &2);
    assert_eq!(transport_metrics.volume, 900);
    assert_eq!(transport_metrics.count, 1);

    let shopping_current = client.get_current_spending(&user, &shopping);
    assert_eq!(shopping_current.volume, 6500);
    assert_eq!(shopping_current.count, 2);

    let shopping_yearly = client.get_yearly_trend(&user, &shopping, &2026);
    assert_eq!(shopping_yearly.volume, 6500);
    assert_eq!(shopping_yearly.count, 2);
}

#[test]
fn test_time_based_filtering() {
    let (env, _admin, client) = setup_analytics_contract();
    let user = Address::generate(&env);
    let category = Symbol::new(&env, "shopping");

    // We will record spending at three different timestamps.
    // Month calculation is based on average month length of 2592000 seconds.
    // Base timestamp = 1768608000 (Feb 2026)
    let t1 = 1768608000;
    env.ledger().set_timestamp(t1);
    client.record_spending(&user, &category, &5000);

    // Still in Feb 2026 but later
    let t2 = 1768608000 + 1000;
    env.ledger().set_timestamp(t2);
    client.record_spending(&user, &category, &3000);

    // Let's check get_category_metrics_filtered
    // 1. Filter including everything
    let filter_all = crate::types::TimeFilter {
        start_timestamp: t1,
        end_timestamp: t2,
    };
    let metrics_all =
        client.get_category_metrics_filtered(&user, &category, &2026, &2, &filter_all);
    assert_eq!(metrics_all.volume, 8000);
    assert_eq!(metrics_all.count, 2);

    // 2. Filter excluding everything (before t1)
    let filter_before = crate::types::TimeFilter {
        start_timestamp: 0,
        end_timestamp: t1 - 1,
    };
    let metrics_before =
        client.get_category_metrics_filtered(&user, &category, &2026, &2, &filter_before);
    assert_eq!(metrics_before.volume, 0);
    assert_eq!(metrics_before.count, 0);

    // 3. Empty/invalid time range (start > end)
    let filter_invalid = crate::types::TimeFilter {
        start_timestamp: t2,
        end_timestamp: t1,
    };
    let metrics_invalid =
        client.get_category_metrics_filtered(&user, &category, &2026, &2, &filter_invalid);
    assert_eq!(metrics_invalid.volume, 0);
    assert_eq!(metrics_invalid.count, 0);

    // 4. Yearly trend filtered including only part of the range
    // Let's query yearly trend filtered with filter_all which covers both transactions
    let yearly_all = client.get_yearly_trend_filtered(&user, &category, &2026, &filter_all);
    assert_eq!(yearly_all.volume, 8000);
    assert_eq!(yearly_all.count, 2);

    // Query yearly trend filtered with filter_before which covers none
    let yearly_before = client.get_yearly_trend_filtered(&user, &category, &2026, &filter_before);
    assert_eq!(yearly_before.volume, 0);
    assert_eq!(yearly_before.count, 0);

    // Query yearly trend filtered with invalid filter
    let yearly_invalid = client.get_yearly_trend_filtered(&user, &category, &2026, &filter_invalid);
    assert_eq!(yearly_invalid.volume, 0);
    assert_eq!(yearly_invalid.count, 0);
}
