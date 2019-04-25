use std::fs;
use std::io;
use std::path::Path;

use minilisp::Interpreter;

#[test]
fn test_lang_suite() -> io::Result<()> {
    for entry in fs::read_dir(Path::new("tests/langsuite/"))? {
        let path = entry?.path();
        assert!(!path.is_dir());

        let mut intp = Interpreter::new().init();
        intp.set_debug_print(true);
        intp.execute_file(&path).expect("Error");
    }
    Ok(())
}
