#[derive(Debug, Clone)]
pub struct DistributionConfig {
    pub treasury_bps: u16,
    pub protocol_bps: u16,
    pub stakeholder_bps: u16,
}

impl DistributionConfig {
    pub fn validate(&self) -> Result<(), &'static str> {
        let total =
            self.treasury_bps +
            self.protocol_bps +
            self.stakeholder_bps;

        if total != 10_000 {
            return Err("distribution percentages must equal 100%");
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct DistributionResult {
    pub treasury: u64,
    pub protocol: u64,
    pub stakeholder: u64,
}

pub fn distribute_fees(
    amount: u64,
    config: &DistributionConfig,
) -> Result<DistributionResult, &'static str> {
    config.validate()?;

    let treasury =
        amount * config.treasury_bps as u64 / 10_000;

    let protocol =
        amount * config.protocol_bps as u64 / 10_000;

    let stakeholder =
        amount - treasury - protocol;

    Ok(DistributionResult {
        treasury,
        protocol,
        stakeholder,
    })
}