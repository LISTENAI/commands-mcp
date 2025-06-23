commands-mcp [![.github/workflows/build.yml](https://github.com/LISTENAI/commands-mcp/actions/workflows/build.yml/badge.svg)](https://github.com/LISTENAI/commands-mcp/actions/workflows/build.yml)
============

[![][npm-version]][npm-url] [![][npm-downloads]][npm-url] [![license][license-img]][license-url] [![issues][issues-img]][issues-url] [![stars][stars-img]][stars-url] [![commits][commits-img]][commits-url]

使用命令模板构建 MCP Server。

<video src="https://github.com/user-attachments/assets/a2800f04-12f6-4c8b-8df7-fee6458739cc"></video>

## 要求

* [Node.js](https://nodejs.org/) 版本 >= 22

## 使用

在你的项目中创建一个 `commands.yaml` 文件，定义你的命令模板。以下是一个用于 Zephyr 项目的示例：

```yaml
# yaml-language-server: $schema=http://listenai.github.io/commands-mcp/schema/master.json

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
    command: |
      source .venv/bin/activate
      west build {{#if pristine}}-p{{/if}} -s {{source_dir}} -b {{board}}

  zephyr_flash:
    description: 将编译好的固件烧录到设备
    command: source .venv/bin/activate && west flash
```

> 推荐配合 [redhat.vscode-yaml](https://marketplace.visualstudio.com/items?itemName=redhat.vscode-yaml) 插件使用，以便提供 YAML 字段补全和验证。

### SSE 模式

大部分 MCP 客户端（如 Claude Desktop、Cline 等）只支持全局的 MCP 服务，因此需要在当前项目中以 SSE 模式运行 commands-mcp，并通过 SSE 接入客户端：

```
$ npx @listenai/commands-mcp
MCP Server is running at http://localhost:53163/sse
Working directory: /home/myproject
```

将上面所输出的 URL 作为 SSE 类型的 MCP Server 填入到你的 MCP 客户端配置中：

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

### STDIO 模式

某些 MCP 客户端支持配置项目级别的 MCP 服务，此时可以直接在项目中以 STDIO 模式启动 commands-mcp。以 GitHub Copilot Chat 为例，编辑 `.vscode/mcp.json`：

```json
{
  "servers": {
    "commands-mcp": {
      "command": "npx",
      "args": [
        "@listenai/commands-mcp",
        "--stdio",
      ],
    },
  },
}
```

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
    * `command` - 执行的命令模板，使用 [Handlebars](https://handlebarsjs.com/guide/expressions.html) 语法
    * `terminate` - 可选，当符合特定条件时自动终止命令执行
      * `timeout` - 命令执行达到指定时间，单位为毫秒
      * `output` - 命令输出（单行）包含特定字符。可使用 `/.../` 语法指定正则表达式（如 `/^return: \d$/`）

## 协议

[Apache-2.0](LICENSE)

[npm-version]: https://img.shields.io/npm/v/@listenai/commands-mcp.svg?style=flat-square
[npm-downloads]: https://img.shields.io/npm/dm/@listenai/commands-mcp.svg?style=flat-square
[npm-url]: https://www.npmjs.org/package/@listenai/commands-mcp
[license-img]: https://img.shields.io/github/license/LISTENAI/commands-mcp?style=flat-square
[license-url]: LICENSE
[issues-img]: https://img.shields.io/github/issues/LISTENAI/commands-mcp?style=flat-square
[issues-url]: https://github.com/LISTENAI/commands-mcp/issues
[stars-img]: https://img.shields.io/github/stars/LISTENAI/commands-mcp?style=flat-square
[stars-url]: https://github.com/LISTENAI/commands-mcp/stargazers
[commits-img]: https://img.shields.io/github/last-commit/LISTENAI/commands-mcp?style=flat-square
[commits-url]: https://github.com/LISTENAI/commands-mcp/commits/master
