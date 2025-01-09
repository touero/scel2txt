use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

struct ScelParser {
    py_table: HashMap<u16, String>,
    g_table: Vec<(u16, String, String)>,
}

impl ScelParser {
    fn new() -> Self {
        ScelParser {
            py_table: HashMap::new(),
            g_table: Vec::new(),
        }
    }

    fn byte_to_str(data: &[u8]) -> String {
        let mut result = String::new();
        for chunk in data.chunks_exact(2) {
            let char_code = u16::from_le_bytes([chunk[0], chunk[1]]);
            if char_code != 0 {
                result.push(char::from_u32(char_code as u32).unwrap_or('?'));
            }
        }
        result
    }

    fn get_py_table(&mut self, data: &[u8]) {
        let mut pos = 0;
        while pos < data.len() {
            let index = u16::from_le_bytes([data[pos], data[pos + 1]]);
            pos += 2;
            let len_py = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2;
            let py = Self::byte_to_str(&data[pos..pos + len_py]);
            self.py_table.insert(index, py);
            pos += len_py;
        }
    }

    fn get_word_py(&self, data: &[u8]) -> String {
        let mut result = String::new();
        for chunk in data.chunks_exact(2) {
            let index = u16::from_le_bytes([chunk[0], chunk[1]]);
            if let Some(py) = self.py_table.get(&index) {
                result.push_str(py);
            }
        }
        result
    }

    fn get_chinese(&mut self, data: &[u8]) {
        let mut pos = 0;
        while pos < data.len() {
            let same = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2;
            let py_table_len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2;
            let py = self.get_word_py(&data[pos..pos + py_table_len]);
            pos += py_table_len;

            for _ in 0..same {
                let c_len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
                pos += 2;
                let word = Self::byte_to_str(&data[pos..pos + c_len]);
                pos += c_len;

                let ext_len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
                pos += 2;
                let count = u16::from_le_bytes([data[pos], data[pos + 1]]);
                pos += ext_len;

                self.g_table.push((count, py.clone(), word));
            }
        }
    }

    fn parse_scel(&mut self, file_path: &Path) {
        let mut file = File::open(file_path).expect("Unable to open file");
        let mut data = Vec::new();
        file.read_to_end(&mut data).expect("Unable to read file");

        println!("---------------------------------------------------------------------------");
        println!("源文件: {}", file_path.to_str().unwrap_or("路径错误"));
        println!("词库名：{}", Self::byte_to_str(&data[0x130..0x338]));
        println!("词库类型：{}", Self::byte_to_str(&data[0x338..0x540]));
        println!("描述信息：{}", Self::byte_to_str(&data[0x540..0xd40]));
        println!("词库示例：{}", Self::byte_to_str(&data[0xd40..0x1540]));

        self.get_py_table(&data[0x1540..0x2628]);
        self.get_chinese(&data[0x2628..]);
    }
}

fn main() {
    let in_path = ".";
    let out_path = "result.txt";

    let mut parser = ScelParser::new();

    let entries = fs::read_dir(in_path).expect("Unable to read directory");
    for entry in entries {
        let entry = entry.expect("Unable to read entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("scel") {
            parser.parse_scel(&path);
        }
    }

    let mut output_file = File::create(out_path).expect("Unable to create output file");
    for (_, _, word) in &parser.g_table {
        writeln!(output_file, "{}", word).expect("Unable to write to file");
    }
    println!("\n\n结果：{}", out_path);
}

