commands-mcp
============

将你的常用命令定义为 MCP 工具，使 AI 代理可以轻松调用。

## 安装

#### NPM

```json
{
  "mcpServers": {
    "commands":{
      "command": "npx",
      "args": [
        "@listenai/commands-mcp@v1"
      ]
    }
  }
}
```

#### Docker

```json
{
  "mcpServers": {
    "commands":{
      "command": "docker",
      "args": [
        "run",
        "--rm",
        "-i",
        "ghcr.io/listenai/commands-mcp:v1"
      ]
    }
  }
}
```

## 使用

在你的项目中创建一个 `commands.yaml` 文件，定义你的命令集合。

示例：

```yaml
# commands.yaml
# yaml-language-server: $schema=https://listenai.github.io/commands-mcp/schema/v1.json

# This file defines the command collection for commands-mcp.
# AI agents should not parse this file directly. Instead, use tools provided by
# commands-mcp, such as `explore_commands`, to access command information.

commands:
  - name: venv_activate
    description: 启用当前项目的 Python 虚拟环境，首次执行 west 命令前必须执行此命令
    command: source {venv_dir}/bin/activate
    args:
      - name: venv_dir
        description: Python 虚拟环境目录，它通常是当前目录下的 .venv
        required: true

  - name: zephyr_build
    description: 编译当前 Zephyr 项目
    args:
      - name: board
        description: 需要构建的 board identifier，如果不能从对话历史中确定，则询问用户
        type: string
        required: true
      - name: source_dir
        description: 项目源码目录，如未指定则使用当前目录
        type: string
    command: west build -p -s {source_dir} -b {board}

  - name: zephyr_flash
    description: 将编译好的固件烧录到设备
    command: west flash
```

> 推荐配合 [redhat.vscode-yaml](https://marketplace.visualstudio.com/items?itemName=redhat.vscode-yaml) 插件使用，以便提供 YAML 字段补全和验证。

## 协议

[Apache-2.0](LICENSE)
