use soroban_sdk::{contract, contractimpl, Env, String};

#[contract]
pub struct VersionContract;

const CONTRACT_VERSION: &str = "1.0.0";

#[contractimpl]
impl VersionContract {
    /// Returns the contract version
    ///
    /// This function does not require authentication.
    pub fn get_version(env: Env) -> String {
        String::from_str(&env, CONTRACT_VERSION)
    }
}#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_get_version_returns_string() {
        let env = Env::default();
        let contract_id = env.register(VersionContract, ());
        let client = VersionContractClient::new(&env, &contract_id);
        let version = client.get_version();
        assert!(!version.is_empty());
    }

    #[test]
    fn test_get_version_format() {
        let env = Env::default();
        let contract_id = env.register(VersionContract, ());
        let client = VersionContractClient::new(&env, &contract_id);
        let version = client.get_version();
        // Version should contain a dot (semver format x.y.z)
        let version_str = version.to_string();
        assert!(version_str.contains('.'));
    }
}
