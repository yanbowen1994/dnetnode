#[derive(Clone, Debug, PartialEq)]
pub struct IptablesRule {
    pub table:          String,
    pub chain:          String,
    pub in_:            String,
    pub out:            String,
    pub src:            String,
    pub dst:            String,
    pub target:         String,
}

impl IptablesRule {
    pub fn new(
        table:          &str,
        chain:          &str,
        in_:            &str,
        out:            &str,
        src:            &str,
        dst:            &str,
        target:         &str,
    ) -> Self {
        IptablesRule {
            table:      table.to_owned(),
            chain:      chain.to_owned(),
            in_:        in_.to_owned(),
            out:        out.to_owned(),
            src:        src.to_owned(),
            dst:        dst.to_owned(),
            target:     target.to_owned(),
        }
    }
}