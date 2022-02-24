# jy-rs
yet another jy copy game using bevy

# 致谢
感谢 https://github.com/wwwjfy/legend-mac 
按照他们的c + lua做的复刻


# 版权声明
本项目采用GPL3协议，用于爱好者的自误自乐和技术交流。

作为复刻版，本项目使用的是金庸群侠传的贴图文件,因为众所周知的原因，我没有把对应的asset放到github上面，请将金庸群侠传里面的 data / pic / sound 目录复制到 assets/org 里面

如果本项目还有其他方面涉及到侵权，联系我，我删

如果有其他法律方面的建议，比如说怎么在不侵权的情况下更好的进行分享，也请联系我。。。

# How it works

1. 安装rust (建议用nightly版本，有很多人民群众需要的功能）
2. 下载代码
  ```
  git clone https://github.com/gaxxx/jy-rs.git
  // 为了方便，我fork了一下bevy
  git clone https://github.com/gaxxx/bevy 
  ```
3. cargo run


# wasm support & fast build

```
cp .cargo/config_fast_build .cargo/config.toml
cargo run --release --target wasm32-unknown-unknown

```
访问 http://127.0.0.1:1334/

在我的机器上，浏览器能到12fps左右。。。后续再优化吧。
![441644538546_ pic](https://user-images.githubusercontent.com/471881/153518664-b4c9d557-dc14-4dd0-ac5b-6a77090a51f3.jpg)


# roadmap
暂时是想到哪做到哪，欢迎提rb

- [x] 完成启动场景的render， 移动和事件 （除了跟娃娃的对话）
- [x] 完成主地图的移动, 重写了canvas
- [ ] 增加主地图的建筑








