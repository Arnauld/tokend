pub fn is_unique_constraint_error(e: &sqlx::Error, contraint_name: Option<&str>) -> bool {
    e.as_database_error()
        .map(|err| {
            if let Some(code) = err.code() {
                if code.eq_ignore_ascii_case("23505") {
                    if let Some(constraint) = contraint_name {
                        // if no constraint is retrieved, one assumes it matches
                        err.constraint()
                            .map(|c| c.eq_ignore_ascii_case(constraint))
                            .unwrap_or(true)
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        })
        .unwrap_or(false)
}
