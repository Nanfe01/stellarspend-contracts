mod support;

use support::setup;

#[test]
fn default_min_fee_is_zero_and_allows_small_amounts() {
    let ctx = setup();
    assert_eq!(ctx.client.get_min_fee(), 0);

    // Collect a tiny fee succeeds when min_fee = 0
    let pending = ctx.client.collect_fee(&ctx.payer, &1i128);
    assert_eq!(pending, 1);
    assert_eq!(ctx.client.get_escrow_balance(), 1);
    assert_eq!(ctx.client.get_pending_fees(&1), 1);
}

#[test]
fn admin_can_update_min_fee_and_equal_or_above_succeeds() {
    let ctx = setup();
    assert_eq!(ctx.client.get_min_fee(), 0);

    // Set min fee to 100
    ctx.client.set_min_fee(&ctx.admin, &100i128);
    assert_eq!(ctx.client.get_min_fee(), 100);

    // Collecting exactly min works
    let pending = ctx.client.collect_fee(&ctx.payer, &100i128);
    assert_eq!(pending, 100);
    assert_eq!(ctx.client.get_escrow_balance(), 100);
    assert_eq!(ctx.client.get_pending_fees(&1), 100);

    // Collecting above min works
    let pending2 = ctx.client.collect_fee(&ctx.payer, &150i128);
    assert_eq!(pending2, 250);
    assert_eq!(ctx.client.get_escrow_balance(), 250);
    assert_eq!(ctx.client.get_pending_fees(&1), 250);
}

#[test]
#[should_panic]
fn collect_below_min_panics() {
    let ctx = setup();
    ctx.client.set_min_fee(&ctx.admin, &100i128);
    // Below min should panic
    ctx.client.collect_fee(&ctx.payer, &50i128);
}

#[test]
#[should_panic]
fn setting_negative_min_fee_is_invalid() {
    let ctx = setup();
    // Negative min_fee should be rejected
    ctx.client.set_min_fee(&ctx.admin, &-1i128);
}

#[test]
#[should_panic]
fn batch_with_item_below_min_panics() {
    let ctx = setup();
    ctx.client.set_min_fee(&ctx.admin, &25i128);

    // Batch where one entry violates min should panic
    let bad_batch = support::amounts(&ctx.env, &[25, 10, 40]);
    ctx.client.collect_fee_batch(&ctx.payer, &bad_batch);
}

#[test]
fn batch_meets_min_succeeds() {
    let ctx = setup();
    ctx.client.set_min_fee(&ctx.admin, &25i128);

    // Batch meeting min per item succeeds
    let ok_batch = support::amounts(&ctx.env, &[25, 30, 40, 100]);
    let res = ctx.client.collect_fee_batch(&ctx.payer, &ok_batch);
    assert_eq!(res.batch_size, 4);
    assert_eq!(res.total_amount, 195);
    assert_eq!(res.cycle, 1);
    assert_eq!(res.pending_fees, 195);
    assert_eq!(ctx.client.get_escrow_balance(), 195);
    assert_eq!(ctx.client.get_pending_fees(&1), 195);
}

#[test]
fn min_fee_works_with_arbitrary_amounts() {
    let ctx = setup();

    // Set a min fee; other discount/cap logic would be applied upstream,
    // this contract simply enforces the final fee is not below min.
    ctx.client.set_min_fee(&ctx.admin, &75i128);

    // Exactly at min
    let p1 = ctx.client.collect_fee(&ctx.payer, &75i128);
    assert_eq!(p1, 75);

    // Above min
    let p2 = ctx.client.collect_fee(&ctx.payer, &200i128);
    assert_eq!(p2, 275);

    assert_eq!(ctx.client.get_escrow_balance(), 275);
    assert_eq!(ctx.client.get_pending_fees(&1), 275);
}

#[test]
fn min_fee_boundary_allows_exact_minimum() {
    let ctx = setup();
    ctx.client.set_min_fee(&ctx.admin, &1i128);

    let pending = ctx.client.collect_fee(&ctx.payer, &1i128);
    assert_eq!(pending, 1);
    assert_eq!(ctx.client.get_escrow_balance(), 1);
    assert_eq!(ctx.client.get_pending_fees(&1), 1);
}
