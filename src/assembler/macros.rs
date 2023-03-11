macro_rules! continue_on_err {
    ($x:ident) => {
        if $x.fails.len() != 0 {
            $x.report();
            continue;
        }
    }
}