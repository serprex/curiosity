use std::process::Command;
use std::collections::HashMap;

#[allow(dead_code)]
pub fn get_root_disk_info() -> HashMap<String, String> {
    // $ /bin/df /
    let output = match Command::new("df").arg("/").output() {
        Ok(output) => output,
        Err(e) => {
            panic!("{}", e);
        }
    };
    
    let stdout = match String::from_utf8(output.stdout) {
        Ok(s) => s.replace("Mounted on", "Mounted_on"),
        Err(e) => {
            panic!("{}", e);
        }
    };
    
    let lines: Vec<&str> = stdout.split("\n").collect();
    let mut df_results: Vec<Vec<&str>> = Vec::new();
    for line in lines.iter() {
        let tokens: Vec<&str> = (*line).split(" ").collect();
        let mut words: Vec<&str> = Vec::new();
        for token in tokens.iter() {
            if token.len() == 0 { continue; }
            words.push(token);
        }
        df_results.push(words);
    }


    let mut stats: Vec<HashMap<String, String>> = Vec::new();
    let mut i = 0;
    for row in df_results.iter() {
        if i == 0 { i += 1; continue; }
        
        let mut stat: HashMap<String, String> = HashMap::new();
        let keys = df_results.get(0).unwrap();
        let mut j = 0;
        for value in row.iter() {
            let mut key = keys.get(j).unwrap().to_string();
            if key == "Mounted_on" { key = "Mounted on".to_string(); };
            stat.insert(key, value.to_string());
            j += 1;
        }

        stats.push(stat);
        i += 1;
    }
    return stats.get(0).unwrap().clone();
}
