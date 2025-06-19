commands-mcp
============

使用命令模板构建 MCP Server。

<video src="https://github.com/user-attachments/assets/a2800f04-12f6-4c8b-8df7-fee6458739cc"></video>

## 要求

* [Node.js](https://nodejs.org/) 版本 >= 22

## 使用

1. 在你的项目中创建一个 `commands.yaml` 文件，定义你的命令模板。以下是一个用于 Zephyr 项目的示例：

    ```yaml
    # yaml-language-server: $schema=http://listenai.github.io/commands-mcp/schema/v1.json

    commands:
      zephyr_build:
        description: 编译当前 Zephyr 项目
        args:
          - name: board
            description: 需要构建的 board identifier，如果不能从对话历史中确定，则询问用户
            type: string
            required: true
          - name: source_dir
            description: 项目源码目录，如未指定则使用当前目录
            type: string
        command: source .venv/bin/activate && west build -p -s {source_dir} -b {board}

      zephyr_flash:
        description: 将编译好的固件烧录到设备
        command: source .venv/bin/activate && west flash
    ```

    > 推荐配合 [redhat.vscode-yaml](https://marketplace.visualstudio.com/items?itemName=redhat.vscode-yaml) 插件使用，以便提供 YAML 字段补全和验证。

2. 在项目目录下运行 MCP Server：

    ```
    $ npx @listenai/commands-mcp
    MCP Server is running at http://localhost:53163/sse
    ```

3. 将上面所输出的 URL 作为 SSE 类型的 MCP Server 填入到你的 MCP 客户端配置中。

    ```json
    {
      "mcpServers": {
        "commands-mcp": {
          "type": "sse",
          "url": "http://localhost:53163/sse"
        }
      }
    }
    ```

    > 如果你的客户端不支持 SSE，可以使用 [mcp-proxy](https://github.com/sparfenyuk/mcp-proxy) 或其它类似的工具转接为 STDIO。

## 配置

* `commands` - 定义命令模板的列表
  * `<name>` - 命令的 tool 名称
    * `description` - 命令的描述
    * `args` - 命令参数列表
      * `name` - 参数名称
      * `description` - 参数描述
      * `type` - 参数类型（如 `string`, `number`, `boolean` 等），默认为 `string`
      * `required` - 是否为必需参数（与 `default` 字段互斥）
      * `default` - 参数的默认值，类型必须与 `type` 字段一致（与 `required` 字段互斥）
    * `command` - 执行的命令模板，支持 `{}` 占位符替换参数

## 协议

[Apache-2.0](LICENSE)
