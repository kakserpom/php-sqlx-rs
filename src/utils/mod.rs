pub mod hashmap_fold;
pub mod ident;
pub mod indent_sql;
pub mod strip_prefix;
pub mod types;

pub fn adhoc_php_class_implements(class: &str, interface: &str) -> Option<()> {
    use ext_php_rs::ffi::zend_do_implement_interface;
    use ext_php_rs::types::ZendStr;
    use ext_php_rs::zend::ClassEntry;
    use ext_php_rs::zend::ExecutorGlobals;
    use std::ptr;
    ExecutorGlobals::get().class_table()?;
    let mut class = ZendStr::new(class, false);
    let interface = ClassEntry::try_find(interface)?;
    unsafe {
        zend_do_implement_interface(
            ext_php_rs::ffi::zend_lookup_class_ex(&raw mut *class, ptr::null_mut(), 0).as_mut()?,
            ptr::from_ref(interface).cast_mut(),
        );
    };
    Some(())
}
