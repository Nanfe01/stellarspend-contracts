use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, Symbol, String, Vec,
};
use crate::{TransactionsContract, TransactionsContractClient, TransactionError, Transaction, TransactionStatus};

#[test]
fn test_initialize_and_get_admin() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    assert_eq!(client.get_admin(), Some(admin.clone()));
}

#[test]
#[should_panic]
fn test_initialize_duplicate_fails() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    let admin2 = Address::generate(&env);
    client.initialize(&admin2);
}

#[test]
fn test_create_transaction() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount: i128 = 1000;
    let note = String::from_str(&env, "Test transaction");
    let memo = String::from_str(&env, "Payment memo");
    let mut tags = Vec::new(&env);
    tags.push_back(String::from_str(&env, "groceries"));
    tags.push_back(String::from_str(&env, "monthly"));

    let tx_id = client.create_transaction(&from, &to, &amount, &note, &memo, &tags);

    let transaction = client.get_transaction(&tx_id).unwrap();
    assert_eq!(transaction.id, tx_id);
    assert_eq!(transaction.from, from);
    assert_eq!(transaction.to, to);
    assert_eq!(transaction.amount, amount);
    assert_eq!(transaction.note, note);
    assert_eq!(transaction.memo, memo);
    assert_eq!(transaction.tags.len(), 2);
    assert_eq!(transaction.tags.get(0), Some(String::from_str(&env, "groceries")));
    assert_eq!(transaction.tags.get(1), Some(String::from_str(&env, "monthly")));
    assert!(transaction.timestamp > 0);
    assert_eq!(transaction.status, crate::TransactionStatus::Completed);
}

#[test]
#[should_panic]
fn test_create_transaction_invalid_amount_zero() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let note = String::from_str(&env, "Invalid amount test");
    let zero_amount: i128 = 0;

    client.create_transaction(&from, &to, &zero_amount, &note, &Vec::new(&env));
}

#[test]
#[should_panic]
fn test_create_transaction_invalid_amount_negative() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let note = String::from_str(&env, "Invalid amount test");
    let negative_amount: i128 = -100;

    client.create_transaction(&from, &to, &negative_amount, &note, &Vec::new(&env));
}

#[test]
fn test_update_transaction_note() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount: i128 = 1000;
    let original_note = String::from_str(&env, "Original note");
    let updated_note = String::from_str(&env, "Updated note");

    let tx_id = client.create_transaction(&from, &to, &amount, &original_note, &Vec::new(&env));

    let transaction = client.get_transaction(&tx_id).unwrap();
    assert_eq!(transaction.note, original_note);

    let success = client.update_transaction_note(&tx_id, &from, &updated_note);
    assert!(success);

    let updated_transaction = client.get_transaction(&tx_id).unwrap();
    assert_eq!(updated_transaction.note, updated_note);
}

#[test]
fn test_update_transaction_amount() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount: i128 = 1000;
    let updated_amount: i128 = 1500;
    let note = String::from_str(&env, "Amount update");
    let tags = Vec::new(&env);

    let tx_id = client.create_transaction(&from, &to, &amount, &note, &tags);

    let success = client.update_transaction_amount(&tx_id, &from, &updated_amount);
    assert!(success);

    let transaction = client.get_transaction(&tx_id).unwrap();
    assert_eq!(transaction.amount, updated_amount);
}

#[test]
#[should_panic]
fn test_transaction_limit_per_user() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let note = String::from_str(&env, "Limit test");
    let one: i128 = 1;

    for _ in 0..1000 {
        let tags = Vec::new(&env);
        client.create_transaction(&from, &to, &one, &note, &tags);
    }

    client.create_transaction(&from, &to, &one, &note, &Vec::new(&env));
}

#[test]
fn test_get_transaction_timestamp() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount: i128 = 1000;
    let note = String::from_str(&env, "Timestamp test");

    let tx_id = client.create_transaction(&from, &to, &amount, &note, &Vec::new(&env));

    let timestamp = client.get_transaction_timestamp(&tx_id);
    assert!(timestamp.is_some());
    assert!(timestamp.unwrap() > 0);

    let fake_id = Symbol::new(&env, "fake_id");
    let fake_timestamp = client.get_transaction_timestamp(&fake_id);
    assert!(fake_timestamp.is_none());
}

#[test]
fn test_get_user_transactions() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let recipient = Address::generate(&env);

    let tx1_id = client.create_transaction(&user1, &recipient, &1000, &String::from_str(&env, "User1 transaction 1"), &Vec::new(&env));
    let tx2_id = client.create_transaction(&user1, &recipient, &2000, &String::from_str(&env, "User1 transaction 2"), &Vec::new(&env));
    let tx3_id = client.create_transaction(&user2, &recipient, &3000, &String::from_str(&env, "User2 transaction"), &Vec::new(&env));

    let user1_txs = client.get_user_transactions(&user1);
    assert_eq!(user1_txs.len(), 2);

    let user2_txs = client.get_user_transactions(&user2);
    assert_eq!(user2_txs.len(), 1);

    let non_existent_user = Address::generate(&env);
    let empty_txs = client.get_user_transactions(&non_existent_user);
    assert_eq!(empty_txs.len(), 0);
}

#[test]
fn test_clear_user_transactions() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let user = Address::generate(&env);
    let recipient = Address::generate(&env);

    let tx1_id = client.create_transaction(&user, &recipient, &1000, &String::from_str(&env, "Transaction 1"), &Vec::new(&env));
    let tx2_id = client.create_transaction(&user, &recipient, &2000, &String::from_str(&env, "Transaction 2"), &Vec::new(&env));

    let user_txs = client.get_user_transactions(&user);
    assert_eq!(user_txs.len(), 2);

    let success = client.clear_user_transactions(&user);
    assert!(success);

    let empty_txs = client.get_user_transactions(&user);
    assert_eq!(empty_txs.len(), 0);

    let tx1 = client.get_transaction(&tx1_id);
    assert!(tx1.is_none());
    let tx2 = client.get_transaction(&tx2_id);
    assert!(tx2.is_none());
}

#[test]
fn test_transaction_counter_increments() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);

    let tx1_id = client.create_transaction(&from, &to, &1000, &String::from_str(&env, "Transaction 1"), &Vec::new(&env));
    let tx2_id = client.create_transaction(&from, &to, &2000, &String::from_str(&env, "Transaction 2"), &Vec::new(&env));
    let tx3_id = client.create_transaction(&from, &to, &3000, &String::from_str(&env, "Transaction 3"), &Vec::new(&env));

    assert_ne!(tx1_id, tx2_id);
    assert_ne!(tx2_id, tx3_id);
    assert_ne!(tx1_id, tx3_id);

    assert!(client.get_transaction(&tx1_id).is_some());
    assert!(client.get_transaction(&tx2_id).is_some());
    assert!(client.get_transaction(&tx3_id).is_some());
}

#[test]
fn test_transaction_exists() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount: i128 = 1000;
    let note = String::from_str(&env, "Existence test");

    let memo = String::from_str(&env, "Existence test memo");
    let tx_id = client.create_transaction(&from, &to, &amount, &note, &memo, &Vec::new(&env));

    assert!(client.transaction_exists(&tx_id));

    let fake_id = Symbol::new(&env, "not_here");
    assert!(!client.transaction_exists(&fake_id));
}

#[test]
fn test_create_transaction_stores_creation_timestamp() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    env.ledger().set_timestamp(1_700_000_123);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let tx_id = client.create_transaction(
        &from,
        &to,
        &500,
        &String::from_str(&env, "timestamped"),
        &Vec::new(&env),
    );

    let tx = client.get_transaction(&tx_id).unwrap();
    assert_eq!(tx.timestamp, 1_700_000_123);
    assert_eq!(client.get_transaction_timestamp(&tx_id), Some(1_700_000_123));
}

#[test]
fn test_get_transaction_memo() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount: i128 = 1000;
    let note = String::from_str(&env, "Test transaction");
    let memo = String::from_str(&env, "Important payment memo");
    let tags = Vec::new(&env);

    let tx_id = client.create_transaction(&from, &to, &amount, &note, &memo, &tags);

    // Test get_transaction_memo function
    let retrieved_memo = client.get_transaction_memo(&tx_id).unwrap();
    assert_eq!(retrieved_memo, memo);
}

#[test]
fn test_get_transaction_memo_nonexistent() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let fake_id = Symbol::new(&env, "not_here");
    
    // Test get_transaction_memo for non-existent transaction
    let memo = client.get_transaction_memo(&fake_id);
    assert!(memo.is_none());
}

#[test]
fn test_delete_transaction_admin_can_remove_record() {
fn test_get_all_transactions() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount: i128 = 1000;
    let note = String::from_str(&env, "Transaction to delete");
    let memo = String::from_str(&env, "Delete memo");

    let tx_id = client.create_transaction(&from, &to, &amount, &note, &memo, &Vec::new(&env));
    assert!(client.transaction_exists(&tx_id));

    let success = client.delete_transaction(&admin, &tx_id);
    assert!(success);
    assert!(!client.transaction_exists(&tx_id));
    assert!(client.get_transaction(&tx_id).is_none());
    assert_eq!(client.get_user_transactions(&from).len(), 0);
}

#[test]
#[should_panic]
fn test_delete_transaction_rejects_non_admin() {
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Create some transactions
    let tx1_id = client.create_transaction(&user1, &recipient, &1000, &String::from_str(&env, "Transaction 1"), &Vec::new(&env));
    let tx2_id = client.create_transaction(&user2, &recipient, &2000, &String::from_str(&env, "Transaction 2"), &Vec::new(&env));
    let tx3_id = client.create_transaction(&user1, &recipient, &3000, &String::from_str(&env, "Transaction 3"), &Vec::new(&env));

    let all_txs = client.get_all_transactions();
    assert_eq!(all_txs.len(), 3);

    // Check that all transactions are present
    let ids: Vec<Symbol> = all_txs.iter().map(|tx| tx.id.clone()).collect();
    assert!(ids.contains(&tx1_id));
    assert!(ids.contains(&tx2_id));
    assert!(ids.contains(&tx3_id));
}

#[test]
fn test_get_all_transactions_empty() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount: i128 = 1000;
    let note = String::from_str(&env, "Transaction to delete");

    let tx_id = client.create_transaction(&from, &to, &amount, &note, &Vec::new(&env));

    let caller = Address::generate(&env);
    client.delete_transaction(&caller, &tx_id);
    let all_txs = client.get_all_transactions();
    assert_eq!(all_txs.len(), 0);
}

#[test]
fn test_get_transactions_paginated_basic() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let note = String::from_str(&env, "tx");
    let memo = String::from_str(&env, "memo");

    for _ in 0..5 {
        client.create_transaction(&from, &to, &100, &note, &memo, &Vec::new(&env));
    }

    // fetch all 5
    let page = client.get_transactions_paginated(&0, &10);
    assert_eq!(page.len(), 5);
}

#[test]
fn test_get_transactions_paginated_offset_and_limit() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let note = String::from_str(&env, "tx");
    let memo = String::from_str(&env, "memo");

    for i in 1_i128..=10 {
        client.create_transaction(&from, &to, &(i * 10), &note, &memo, &Vec::new(&env));
    }

    // page 1: offset=0, limit=3 → first 3
    let page1 = client.get_transactions_paginated(&0, &3);
    assert_eq!(page1.len(), 3);
    assert_eq!(page1.get(0).unwrap().amount, 10);
    assert_eq!(page1.get(2).unwrap().amount, 30);

    // page 2: offset=3, limit=3 → next 3
    let page2 = client.get_transactions_paginated(&3, &3);
    assert_eq!(page2.len(), 3);
    assert_eq!(page2.get(0).unwrap().amount, 40);

    // last page: offset=9, limit=5 → only 1 remaining
    let last = client.get_transactions_paginated(&9, &5);
    assert_eq!(last.len(), 1);
    assert_eq!(last.get(0).unwrap().amount, 100);
}

#[test]
fn test_get_transactions_paginated_offset_beyond_total() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let note = String::from_str(&env, "tx");
    let memo = String::from_str(&env, "memo");
    client.create_transaction(&from, &to, &100, &note, &memo, &Vec::new(&env));

    let page = client.get_transactions_paginated(&10, &5);
    assert_eq!(page.len(), 0);
}

#[test]
fn test_get_transactions_paginated_limit_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let note = String::from_str(&env, "tx");
    let memo = String::from_str(&env, "memo");
    client.create_transaction(&from, &to, &100, &note, &memo, &Vec::new(&env));

    let page = client.get_transactions_paginated(&0, &0);
    assert_eq!(page.len(), 0);
}

#[test]
fn test_get_transactions_paginated_limit_capped_at_100() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let note = String::from_str(&env, "tx");
    let memo = String::from_str(&env, "memo");

    for _ in 0..120 {
        client.create_transaction(&from, &to, &1, &note, &memo, &Vec::new(&env));
    }

    // requesting 200 should be capped to 100
    let page = client.get_transactions_paginated(&0, &200);
    assert_eq!(page.len(), 100);
}

#[test]
fn test_get_user_transactions_filtered_by_tx_type() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let user = Address::generate(&env);
    let to = Address::generate(&env);
    let note = String::from_str(&env, "typed");
    let memo = String::from_str(&env, "memo");
    let tags = Vec::new(&env);
    let income = Symbol::new(&env, "income");
    let expense = Symbol::new(&env, "expense");

    let income_tx_1 = client.create_transaction(&user, &to, &100, &note, &memo, &tags, &income);
    let expense_tx = client.create_transaction(&user, &to, &50, &note, &memo, &tags, &expense);
    let income_tx_2 = client.create_transaction(&user, &to, &75, &note, &memo, &tags, &income);

    let income_txs = client.get_user_transactions_filtered(&user, &income);
    assert_eq!(income_txs.len(), 2);
    assert_eq!(income_txs.get(0).unwrap().id, income_tx_1);
    assert_eq!(income_txs.get(1).unwrap().id, income_tx_2);

    let expense_txs = client.get_user_transactions_filtered(&user, &expense);
    assert_eq!(expense_txs.len(), 1);
    assert_eq!(expense_txs.get(0).unwrap().id, expense_tx);
}

#[test]
fn test_get_total_expense() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let user = Address::generate(&env);
    let to = Address::generate(&env);
    let note = String::from_str(&env, "note");
    let memo = String::from_str(&env, "memo");
    let tags = Vec::new(&env);
    let income = Symbol::new(&env, "income");
    let expense = Symbol::new(&env, "expense");

    // No transactions yet
    assert_eq!(client.get_total_expense(), 0);

    client.create_transaction(&user, &to, &200, &note, &memo, &tags, &income);
    client.create_transaction(&user, &to, &50, &note, &memo, &tags, &expense);
    client.create_transaction(&user, &to, &30, &note, &memo, &tags, &expense);

    // Only expense amounts should be summed
    assert_eq!(client.get_total_expense(), 80);
}