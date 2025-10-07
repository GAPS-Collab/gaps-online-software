(function() {
    var type_impls = Object.fromEntries([["polars",[]],["polars_core",[]],["polars_ops",[]],["polars_plan",[]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[13,19,18,19]}