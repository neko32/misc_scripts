#![warn(clippy::all, clippy::pedantic)]

pub mod update_check {

    use std::fs::File;
    use std::io::Read;
    use std::process::Command;
    use regex::Regex;
    use curl::easy::{List, Easy, Auth};
    use json::object;

    /// # Panics
    /// 
    /// Will panic if re.find() goes south
    /// # Errors
    /// 
    /// May bubble up errors
    pub fn run_rustup_chk() -> core::result::Result<(bool, String), anyhow::Error> {
        let rez = Command::new("rustup")
        .arg("check")
        .output()?;
        let out = rez.stdout;
        let str_out = String::from_utf8(out)?;
        let sout_str = str_out.as_str();
        println!("rustup check output was :: {}", &str_out);
        let is_up_to_date = str_out.contains("Up to date");
        let re = Regex::new(r"\d+\.\d+\.\d+")?;
        let ver = re.find(sout_str).unwrap().as_str();
        if is_up_to_date {
            Ok((true, ver.to_string()))
        } else {
            Ok((false, ver.to_string()))
        }
    }

    /// # Errors
    /// 
    /// This function may bubble up errors
    pub fn create_jira_backlog(conf_store:&String, version:&String) -> core::result::Result<(), anyhow::Error> {
        // prep
        let conf = load_resource(conf_store)?;
        let mut curl = Easy::new();
        println!("{:?}", conf);
        
        // auth
        let mut auth = Auth::new();
        auth.basic(true);
        curl.http_auth(&auth)?;
        curl.username(conf.user.as_str())?;
        curl.password(conf.cred.as_str())?;

        // header
        let mut header_list = List::new();
        header_list.append("Content-Type: application/json")?;
        curl.url(conf.url.as_str())?;
        curl.http_headers(header_list)?;

        // payload
        let payload = gen_req_payload(conf, version);
        curl.post_fields_copy(payload.as_bytes())?;
        println!("payload is {}", &payload);
        

        // transmission
        curl.post(true)?;
        let mut resp:String = String::new();
        {
            let mut txfr = curl.transfer();
            txfr.write_function(|data| {
                resp = String::from_utf8_lossy(data).to_string();
                Ok(data.len())
            })?;
            txfr.perform()?;
        }

        // response handling
        let resp_code = curl.response_code()?;
        println!("{}", resp_code);
        let resp_json = json::parse(resp.as_str())?;
        println!("received response from JIRA --- response code {}", resp_code);
        println!("{}", json::stringify_pretty(resp_json, 2));
        Ok(())
    }

    fn gen_req_payload(conf:Conf, version:&String) -> String {

        let payload_json = object! {
            "fields" => object! {
                "project" => object! {
                    "key" => conf.key,
                },
                "summary" => format!("Upgrade Rust to version {}", version),
                "description" => format!("As a result of rustup check, latest version {} was found. Please update by running rustup update", version),
                "issuetype" => object! {
                    "name" => "Story"
                }
            }
        };
        payload_json.to_string()
                
    }

    fn load_resource(conf_store:&String) -> core::result::Result<Conf, anyhow::Error> {
        let mut fpath = File::open(conf_store)?;
        let mut buf:String = String::new();
        fpath.read_to_string(&mut buf)?;
        let json_conf = json::parse(buf.as_str())?;
        Ok(
            Conf {
                url:json_conf["url"].as_str().unwrap().to_string(),
                key:json_conf["key"].as_str().unwrap().to_string(),
                user:json_conf["user"].as_str().unwrap().to_string(),
                cred:json_conf["cred"].as_str().unwrap().to_string(),
            }
        )
    }

    #[derive(Debug)]
    struct Conf {
        url:String,
        key:String,
        user:String,
        cred:String
    }


}
