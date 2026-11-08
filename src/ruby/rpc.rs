use crate::diag::DiagError;
use crate::ruby::runner::ensure_ruby_available;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub struct RpcClient {
    plugin_name: String,
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
    request_id: u64,
}

impl RpcClient {
    pub fn new(plugin_name: &str, script_path: &Path, plugin_dir: &Path) -> Result<Self, DiagError> {
        ensure_ruby_available()?;
        let mut child = Command::new("ruby")
            .arg(script_path)
            .current_dir(plugin_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| DiagError::from_code("E033", &[("name", plugin_name), ("reason", &e.to_string())]))?;

        let stdin = child.stdin.take()
            .ok_or_else(|| DiagError::from_code("E033", &[("name", plugin_name), ("reason", "failed to open stdin")]))?;
        let stdout = child.stdout.take()
            .ok_or_else(|| DiagError::from_code("E033", &[("name", plugin_name), ("reason", "failed to open stdout")]))?;

        Ok(Self {
            plugin_name: plugin_name.to_string(),
            child,
            stdin,
            reader: BufReader::new(stdout),
            request_id: 0,
        })
    }

    pub fn call(&mut self, method: &str, params: Value) -> Result<Value, DiagError> {
        self.request_id += 1;
        let req = json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params,
        });
        let req_str = json_to_line(&req)?;
        self.stdin.write_all(req_str.as_bytes())
            .map_err(|e| DiagError::from_code("E033", &[("name", &self.plugin_name), ("reason", &e.to_string())]))?;
        self.stdin.flush()
            .map_err(|e| DiagError::from_code("E033", &[("name", &self.plugin_name), ("reason", &e.to_string())]))?;

        let mut line = String::new();
        self.reader.read_line(&mut line)
            .map_err(|e| DiagError::from_code("E033", &[("name", &self.plugin_name), ("reason", &e.to_string())]))?;
        if line.trim().is_empty() {
            return Err(DiagError::from_code("E033", &[("name", &self.plugin_name), ("reason", "empty response from plugin")]));
        }

        let resp: Value = serde_json::from_str(&line)
            .map_err(|e| DiagError::from_code("E033", &[("name", &self.plugin_name), ("reason", &e.to_string())]))?;
        if let Some(err_obj) = resp.get("error") {
            let msg = err_obj.get("message").and_then(|m| m.as_str()).unwrap_or("unknown error");
            return Err(DiagError::from_code("E033", &[("name", &self.plugin_name), ("reason", msg)]));
        }
        resp.get("result")
            .cloned()
            .ok_or_else(|| DiagError::from_code("E033", &[("name", &self.plugin_name), ("reason", "missing result in response")]))
    }
}

fn json_to_line(v: &Value) -> Result<String, DiagError> {
    let mut s = serde_json::to_string(v)
        .map_err(|e| DiagError::new("E033", e.to_string()))?;
    s.push('\n');
    Ok(s)
}

impl Drop for RpcClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_roundtrip_with_ruby() {
        let tmp = std::env::temp_dir().join("lacie_test_rpc");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let script_path = tmp.join("server.rb");
        let ruby_code = r#"
require 'json'
$stdout.sync = true
while (line = $stdin.gets)
  req = JSON.parse(line.strip)
  if req['method'] == 'ping'
    puts JSON.generate({'jsonrpc' => '2.0', 'id' => req['id'], 'result' => 'pong'})
  else
    puts JSON.generate({'jsonrpc' => '2.0', 'id' => req['id'], 'error' => {'code' => -32601, 'message' => 'not found'}})
  end
end
"#;
        std::fs::write(&script_path, ruby_code).unwrap();
        let mut client = RpcClient::new("test", &script_path, &tmp).unwrap();
        let res = client.call("ping", json!({})).unwrap();
        assert_eq!(res.as_str(), Some("pong"));

        let err = client.call("other", json!({})).unwrap_err();
        assert_eq!(err.code, "E033");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
