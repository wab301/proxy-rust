介绍
====

本项目是一个基于[`funny/proxy`](https://github.com/funny/proxy)使用Rust重制的通用网关程序。

本网关只有TCP流量转发功能，负责为每个客户端连接建立一个后端连接进行流量转发。

本网关有以下用途：

1. 避免应用服务器直接暴露到公网
2. 提高故障转移的效率

本网关有以下特性：

1. 易接入，只需要对客户端做小量修改即可接入，不需要修改已又通讯协议
2. 可扩展，可以任意多开水平扩展以实现负载均衡和高可
3. 零配置，运维人员无需手工进行后端服务器列表配置

协议
====

客户端连接网关后，发送一行`base64`编码过的服务器地址密文到网关，并等待网关回发状态码。

可能收到的状态码如下：

| 状态码 | 状态说明 |
|-----|---------|
| 200 | 握手完成，可以开始传输数据 |
| 400 | 请求数据读取过程中发生错误 |
| 401 | 网关解密地址信息失败 |
| 502 | 网关无法连接后端服务器 |
| 504 | 网关连接后端服务器超时 |

客户端收到成功状态后，即可开始和目标服务器进行通讯了。

基本通信流程：

1. 客户端连接网关
2. 客户端发送目标服务器地址密文
    * 如果读取失败，回发`400`状态码给客户端
3. 网关解密目标服务器地址
    * 如果解密失败，回发`401`状态码给客户端
4. 网关连接目标服务器
    * 如果发生错误，回发`502`状态码给客户端
    * 如果发生超时，回发`504`状态码给客户端
5. 网关回发成功状态码`200`给客户端
6. 网关发送缓存中残余数据给目标服务器
7. 客户端和目标服务器之间开始对传数据

加密
====

客户端发送到网关的目标服务器地址使用`AES256-CBC`加密并进行`base64`编码，密文以换行符结尾。

示例：

```
U2FsdGVkX19KIJ9OQJKT/yHGMrS+5SsBAAjetomptQ0=\n
```

进行加密目的是让外网攻击者无法对网关后的内网服务器进行猜测和任意连接。

接入流程：

1. 生成`Secret`，并保存在安全的文档中
  * 可以在线生成：<https://lastpass.com/generatepassword.php>
2. 使用上述`Secret`部署网关
3. 使用`AES`算法加密文本格式的后端地址，生成`base64`编码的密文
    * 可以在线生成：<http://tool.oschina.net/encrypt>
    * 也可以使用`openssl`命令生成，如：

    ```
    echo -n "127.0.0.1:62863" | openssl enc -e -aes-256-cbc -a -salt -k "p0S8rX680*48"
    ```

    * 举例，当后端地址为`127.0.0.1:62863`并且`Secret`为`p0S8rX680*48`时，密文结果应类似：

    ```
    U2FsdGVkX19KIJ9OQJKT/yHGMrS+5SsBAAjetomptQ0=
    ```

    _注：上述方式都会使用随机Salt，这也是建议的方式。其结果是每次加密得出的密文结果并不一样，但并不会影响解密_

加密后的服务器地址通常是在拉取服务器列表的场景中发送给客户端，客户端只会有加密后的地址，不应该有`Secret`或服务器明文地址。

重要的事情说三遍：

* 切勿将`Secret`写入客户端代码！
* 切勿将`Secret`写入客户端代码！
* 切勿将`Secret`写入客户端代码！

设置
====

网关可以通过以下命令行参数进行设置：

| 变量 | 用途 |
|-----|----|
| `secret` | 解密地址用的秘钥，必须设置 |
| `addr` | 网关服务器地址，默认为0.0.0.0:6001 |
| `retry` | 网关连接目标服务器的重试次数，默认为1 |
| `timeout` | 网关每次连接目标服务器的超时时间，单位是秒，默认为3 |
