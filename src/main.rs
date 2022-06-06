use std::io::Read;

use async_std::task::block_on;
use serde::Deserialize;
use urlencoding::encode;

#[derive(Deserialize)]
struct Config {
    // 网络服务商
    pub isp: String,
    pub username: String,
    pub password: String,
}

fn main() {
    let login_url = get_login_url();
    println!("{}", login_url);
    let login_data = get_login_data();
    println!("{}", login_data);
    let post = surf::post(login_url)
        .body_string(login_data)
        .header("Connection", "keep-alive")
        .header("Cache-Control", "max-age=0")
        .header("Upgrade-Insecure-Requests", "1")
        .header("Origin", "http://10.10.244.11")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9")
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.93 Safari/537.36 Edg/96.0.1054.53");
    let _res = block_on(post).unwrap();
    // TODO: 添加结果检测
}

/// 读取配置，生成连接数据
fn get_login_data() -> String {
    let cfg = std::env::current_exe().unwrap().parent().unwrap().join("cfg.toml");
    let mut cfg = std::fs::OpenOptions::new().read(true).open(cfg).expect("读取配置文件时发生错误");
    let mut cfg_buf = Vec::new();
    cfg.read_to_end(&mut cfg_buf).unwrap();
    let cfg: Config = toml::from_slice(cfg_buf.as_slice()).unwrap();
    format!(
        "DDDDD={}&upass={}&R1=0&R2=0&R3=0&R6=0&para=00&0MKKey=123456&buttonClicked=&redirect_url=&err_flag=&username=&password=&user=&cmd=&Login=&v6ip=",
        encode(&format!("{}{}", cfg.username, if cfg.isp.is_empty() {cfg.isp} else {format!("@{}", cfg.isp)})),
        encode(&cfg.password) // 密码和帐号信息要url编码，但其他的不用
    )
}

/// 通过重定向获取网络参数(网卡分配到的ip，学校路由ip、名称)，进而获取登录URL
fn get_login_url() -> surf::Url {
    // GET 6.6.6.6会重定向到登录界面url,url末尾有上述参数
    let recv = block_on(surf::get("http://6.6.6.6:80").recv_string()).expect("连接时发生错误，请检查网络连接");
    // 截取参数(wlanuserip, wlanacip, wlanacname)
    let wlan_info = recv.to_owned().split_once("?").unwrap().1.split_once('"').unwrap().0.to_owned();
    let ip = wlan_info.split('&').filter(|p| p.contains("wlanuserip")).next().unwrap().split_once("=").unwrap().1;
    let mut base_url = String::from("http://10.10.244.11:801/eportal/?c=ACSetting&a=Login&protocol=http:&hostname=10.10.244.11&iTermType=1&");
    surf::Url::parse_with_params(
        { base_url.push_str(&wlan_info); &base_url },
        &[("mac","00-00-00-00-00-00"), ("ip", ip), ("enAdvert", "0"), ("queryACIP", "0"), ("loginMethod", "1")]
    ).unwrap()
}

/// 获取当前网卡的局域网ip(v4)，调试用
#[inline]
fn _get_local_ipv4() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    // 下面的地址实际上可以随便填(有效范围内)
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}