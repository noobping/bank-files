use super::*;
use std::collections::HashSet;

pub fn budget_rollup_codes(budgets: &[BudgetCode], root_code: &str) -> Vec<String> {
    let root_code = root_code.trim();
    if root_code.is_empty() {
        return Vec::new();
    }

    let mut codes = vec![root_code.to_string()];
    let mut visited = HashSet::from([budget_code_key(root_code)]);
    let mut index = 0;
    while index < codes.len() {
        let parent = codes[index].clone();
        index += 1;
        for budget in budgets
            .iter()
            .filter(|budget| budget.parent_code.trim().eq_ignore_ascii_case(&parent))
        {
            let key = budget_code_key(&budget.code);
            if visited.insert(key) {
                codes.push(budget.code.clone());
            }
        }
    }

    codes
}

pub fn budget_has_children(budgets: &[BudgetCode], code: &str) -> bool {
    let code = code.trim();
    !code.is_empty()
        && budgets
            .iter()
            .any(|budget| budget.parent_code.trim().eq_ignore_ascii_case(code))
}

pub fn budget_code_matches_tree(budgets: &[BudgetCode], root_code: &str, tx_code: &str) -> bool {
    let tx_code = tx_code.trim();
    if tx_code.eq_ignore_ascii_case(root_code.trim()) {
        return true;
    }
    budget_rollup_codes(budgets, root_code)
        .into_iter()
        .any(|code| code.trim().eq_ignore_ascii_case(tx_code))
}

fn budget_code_key(code: &str) -> String {
    code.trim().to_ascii_lowercase()
}
