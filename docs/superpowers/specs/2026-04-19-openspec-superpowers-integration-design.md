# 设计文档：OpenSpec + Superpowers 集成方案

**日期：** 2026-04-19
**项目：** Courier（内网穿透隧道管理系统）
**目标：** 设计 OpenSpec（SDD 规范治理层）与 Superpowers（工程执行质量层）的完整集成方案，覆盖软件开发全生命周期

## 实施状态

| 项目 | 状态 | 说明 |
|---|---|---|
| OpenSpec CLI 安装 | ✅ 已完成 | v1.3.0，`npm install -g @fission-ai/openspec@latest` |
| `openspec init --tools claude` | ✅ 已完成 | 在 Courier 项目执行，生成 `.claude/commands/opsx/` 和 `.claude/skills/openspec-*` |
| 用户级集成规则 | ✅ 已完成 | `~/.claude/CLAUDE.md`（全局生效，不修改项目 CLAUDE.md） |

---

## 背景

### 两个框架的定位

| 框架 | 层次 | 解决的问题 | 核心产物 |
|---|---|---|---|
| **OpenSpec**（Fission-AI） | 规范治理层 | **构建什么**（What to build） | `openspec/changes/<id>/`：proposal.md、specs/、design.md、tasks.md |
| **Superpowers**（obra） | 工程执行层 | **正确地构建**（Build it right） | brainstorming 对话、TDD 测试、subagent 执行、两阶段代码审查 |

### 现状与问题

Courier 项目目前已有 Superpowers 工作流（CLAUDE.md + rules/），但缺少 OpenSpec 规范治理层，导致：
- 大功能需求靠对话记忆，无持久化规格文档
- 代码审查缺乏「行为规格」参照基准，只能审查编码风格
- 功能完成后无法追溯「当初为什么这样设计」

### 选择方案：C-3（已修订）

经过分析三种集成路径，选择 **C-3：brainstorming 驱动，结合 `/opsx:propose` CLI 生成 OpenSpec artifacts**。

实际安装 OpenSpec v1.3.0 后，发现 `/opsx:propose` 命令通过 `openspec new change` + `openspec instructions` CLI 工具生成结构化 artifact 模板，比纯手写格式更规范。因此修订 C-3：

- **brainstorming 在前**：通过 `superpowers:brainstorming` 对话完成需求探索和方案确认
- **`/opsx:propose` 在后**：brainstorming 结束后，用 `/opsx:propose` 生成 OpenSpec 目录结构，Claude 将 brainstorming 产出填入各 artifact
- 两个框架通过 `openspec/changes/<id>/` 目录约定衔接
- 集成规则放在**用户级** `~/.claude/CLAUDE.md`，对所有项目全局生效，不修改各项目的 CLAUDE.md

---

## 适用范围：小改动 vs 大功能

### 判断标准

| 维度 | 小改动 | 大功能 |
|---|---|---|
| 新增代码量 | ≤50 行 | >50 行 |
| 涉及模块 | 单文件或局部修改 | 跨模块/多文件 |
| 行为变化 | 修复已有行为 | 引入新行为或新接口 |
| 需求明确性 | 用户描述已足够清晰，无需澄清 | 需要对话探索目的与约束 |

**升级触发条件**（小改动途中发现以下情况，切换为大功能流程）：
- 实现范围超出预期，影响多个模块
- 需要新增 Cargo.toml 或 package.json 依赖
- 影响已有 API 契约（修改响应结构、删除字段等）

### 两条路径总览

```
用户提需求
    │
    ├─ 小改动（≤50行，单模块，修复行为）
    │       │
    │       ├─ 读相关文件，明确改动范围
    │       ├─ TDD：写失败测试 → 实现 → 测试通过
    │       ├─ git commit
    │       └─ 无需 OpenSpec 文档
    │
    └─ 大功能（>50行，跨模块，新行为）
            │
            └─ 走完整 9 阶段流程（见下文）
```

---

## 大功能：完整 9 阶段集成流程

### 目录结构约定

```
openspec/
└── changes/
    ├── <change-id>/           ← 进行中的变更（如 feat-jwt-auth）
    │   ├── proposal.md        ← Phase 1 产物：需求摘要
    │   ├── specs/             ← Phase 2 产物：行为规格（GIVEN/WHEN/THEN）
    │   │   ├── 01-core-behavior.md
    │   │   └── 02-edge-cases.md
    │   ├── design.md          ← Phase 3 产物：技术设计
    │   └── tasks.md           ← Phase 4 产物：任务状态跟踪
    └── archive/               ← 已完成变更归档
        └── <change-id>/
```

---

### Phase 1：需求捕获

**工具：** `superpowers:brainstorming`

**流程：**
1. 用户描述需求意图
2. brainstorming 技能一问一答，探索目的、约束、成功标准、排除项
3. 对话结束，用户确认方向无误
4. 执行 `/opsx:propose <change-id>` → CLI 生成 `openspec/changes/<id>/` 目录结构
5. Claude 将 brainstorming 产出填入 CLI 生成的 proposal.md 模板

**落地可行性：✅ 完全可行**
- brainstorming 内置「探索项目上下文 → 澄清问题 → 提出方案」流程，直接支持
- `/opsx:propose` CLI 已安装（v1.3.0），`openspec new change` 自动创建目录和 `.openspec.yaml`
- `openspec instructions proposal --change "<id>" --json` 提供标准模板，Claude 填充内容

**proposal.md 格式（由 openspec instructions 提供模板）：**
```markdown
# 变更提案：<功能名>

**变更 ID：** <change-id>
**日期：** YYYY-MM-DD
**状态：** draft | approved | in-progress | done

## 需求摘要
[1-3 句话描述要构建什么]

## 成功标准
- [ ] 标准 1
- [ ] 标准 2

## 排除项（不在本次范围内）
- 排除 A

## 背景与动机
[为什么需要这个功能]
```

---

### Phase 2：行为规格（Behavior Specs）

**工具：** brainstorming 对话尾声 → Claude 写文件

**流程：**
1. brainstorming 对话中确认核心场景后
2. Claude 将每个场景转为 GIVEN/WHEN/THEN 格式
3. 生成 `openspec/changes/<id>/specs/` 下的规格文件
4. 用户在审阅 proposal.md 时一并确认 specs/

**落地可行性：✅ 可行（需 Claude 主动引导）**
- GIVEN/WHEN/THEN 是标准 BDD 格式，Claude 可自然从对话中提取
- 摩擦点：用户可能跳过场景确认步骤；缓解：CLAUDE.md 规则要求 Claude 在 brainstorming 结束前主动生成规格草稿供用户确认
- specs/ 是审查阶段的核心依据，写得越具体，后续验证越容易

**specs 文件格式：**
```markdown
# 行为规格：<模块名>

## 场景：<场景名>

**前置条件（GIVEN）**
- 系统状态 A
- 输入条件 B

**触发动作（WHEN）**
- 用户/系统执行操作 C

**预期结果（THEN）**
- 系统返回 D
- 状态变更为 E
- 错误场景：若 F，则返回错误码 G
```

---

### Phase 3：技术设计

**工具：** brainstorming（方案讨论部分）→ Claude 写文件

**流程：**
1. brainstorming 提出 2-3 个实现方案，分析权衡
2. 用户确认选择的方案
3. Claude 将确认方案写入 `openspec/changes/<id>/design.md`
4. 内容包括：模块职责变更、数据结构/接口定义、关键决策记录

**落地可行性：✅ 完全可行**
- brainstorming 技能本身包含「Propose 2-3 approaches with trade-offs」步骤
- design.md 自由格式，无 schema 约束
- 关键：design.md 中的接口定义要与 specs/ 中的 GIVEN/WHEN/THEN 一致

---

### Phase 4：任务拆解

**工具：** `superpowers:writing-plans`

**流程：**
1. writing-plans 读取 `proposal.md` + `specs/` + `design.md`
2. 生成含完整 TDD 步骤的详细任务计划
3. **同时写两份文件：**
   - `docs/superpowers/plans/YYYY-MM-DD-<name>.md`：执行用，含完整代码块和命令
   - `openspec/changes/<id>/tasks.md`：治理用，仅任务列表与状态

**落地可行性：✅ 可行（有轻量桥接工作）**
- writing-plans 输出格式已非常详细，直接可用
- tasks.md 是轻量摘要版，Claude 在生成 plans/ 后顺手提取任务列表即可

**tasks.md 格式：**
```markdown
# 任务列表：<功能名>

| # | 任务 | 状态 | 详细计划 |
|---|---|---|---|
| 1 | 实现 X 接口 | ⏳ pending | docs/superpowers/plans/...#task-1 |
| 2 | 添加数据库迁移 | ⏳ pending | docs/superpowers/plans/...#task-2 |
| 3 | 集成测试 | ⏳ pending | docs/superpowers/plans/...#task-3 |

**状态图例：** ⏳ pending · 🔄 in-progress · ✅ done · ❌ blocked
```

---

### Phase 5：单元测试（TDD）

**工具：** `superpowers:test-driven-development`（通过 subagent 执行）

**流程：**
每个任务强制 RED → GREEN → REFACTOR：
1. 写失败测试（描述预期行为）
2. 运行确认失败（验证测试有效）
3. 写最小实现使测试通过
4. 运行确认通过
5. 提交

**落地可行性：✅ 完全可行**
- writing-plans 生成的任务已内置完整 TDD 步骤（Step 1: 写失败测试 / Step 2: 运行确认失败 ...）
- subagent-driven-development 要求 subagent 遵循 TDD，无额外配置

---

### Phase 6：编码实现

**工具：** `superpowers:subagent-driven-development`

**流程：**
1. Controller 读取 plans/ 文件，逐任务派发新 subagent（隔离上下文）
2. subagent prompt 包含：
   - 完整任务文本（含代码块）
   - 对应 specs/ 中的行为规格（作为验收标准）
   - `.claude/rules/` 相关约束提示
3. subagent 实现 → 自检 → 提交
4. Controller 不进入下一任务，直至当前任务两阶段审查通过

**落地可行性：✅ 完全可行**
- 关键改进：subagent prompt 注入 specs/ 规格，让 subagent 在实现时就对齐验收标准
- 每任务独立 commit，便于追溯

---

### Phase 7：两阶段代码审查

**工具：** `superpowers:subagent-driven-development`（内置审查循环）

**第一阶段：规格合规审查（Spec Reviewer）**
- 对照 `openspec/changes/<id>/specs/` 中每条 GIVEN/WHEN/THEN
- 验证实现覆盖所有场景，无遗漏，无超出范围的额外行为
- 通过标准：每条规格可追溯到对应的测试或实现代码

**第二阶段：代码质量审查（Code Quality Reviewer）**
- 对照 `.claude/rules/02-rust-conventions.md`（或 03-frontend-conventions.md）
- 检查命名、错误处理、日志、无 `.unwrap()`、无调试代码
- 通过标准：无 Important 或 Critical 级别问题

**落地可行性：✅ 可行（比原版更强）**
- 原版 spec reviewer 对照 plans/ 文件；集成后改为对照 `specs/*.md`，基准更精准
- 两阶段分离：「功能是否正确」与「实现是否规范」独立评判，互不干扰

---

### Phase 8：验证

**工具：** Claude 直接执行（无工具依赖）

**流程：**
1. 所有任务完成后，Claude 逐条核查 `specs/*.md` 中的 GIVEN/WHEN/THEN
2. 运行 `cargo test --workspace` 作为自动验证
3. 更新 `openspec/changes/<id>/tasks.md` 所有任务状态为 ✅
4. 更新 `proposal.md` 中 `状态` 字段为 `done`

**落地可行性：✅ 可行**
- specs/ 写得越具体，验证越高效；行为规格即验收标准
- 无需额外工具

---

### Phase 9：归档

**工具：** git + 文件操作

```bash
mkdir -p openspec/changes/archive/
git mv openspec/changes/<id>/ openspec/changes/archive/<id>/
git add openspec/changes/archive/<id>/
git commit -m "chore: 归档 openspec 变更 <id>"
```

**落地可行性：✅ 完全可行**

---

## CLAUDE.md 落地方式

### 用户级（推荐，本方案采用）

规则放入 `~/.claude/CLAUDE.md`，对该用户所有项目全局生效：
- **优点：** 无需修改各项目 CLAUDE.md；换项目自动携带工作流
- **适用：** 工作流是开发者个人习惯，而非项目约束

### 项目级（备选）

规则放入项目根目录 `CLAUDE.md`，仅对该项目生效：
- **适用：** 团队所有成员都需遵循该工作流，且项目强制要求 SDD

**本方案选择用户级落地。**

---

## 用户级集成规则（~/.claude/CLAUDE.md）

写入 `~/.claude/CLAUDE.md`，内容如下：

**判断路径：**
- 新增代码 ≤50 行、单模块修改、修复已有行为 → **小改动路径**（直接 TDD + 实现，无需 OpenSpec 文档）
- 新增代码 >50 行、跨模块、引入新行为 → **大功能路径**（走 9 阶段流程）

**大功能：9 阶段流程**

| 阶段 | 工具 | 产物 |
|---|---|---|
| 1 需求捕获 | superpowers:brainstorming | openspec/changes/<id>/proposal.md |
| 2 行为规格 | brainstorming 尾声 | openspec/changes/<id>/specs/ |
| 3 技术设计 | brainstorming 方案讨论 | openspec/changes/<id>/design.md |
| 4 任务拆解 | superpowers:writing-plans | openspec/changes/<id>/tasks.md + docs/superpowers/plans/ |
| 5 单元测试 | TDD（subagent 执行） | 测试文件 + git commits |
| 6 编码实现 | superpowers:subagent-driven-development | git commits |
| 7 代码审查 | 两阶段 review（规格合规 + 代码质量） | review 结论 |
| 8 验证 | 对照 specs/ 逐条核查 | tasks.md 全部 ✅ |
| 9 归档 | git mv | openspec/changes/archive/<id>/ |

**规格审查基准：** openspec/changes/<id>/specs/ 中的 GIVEN/WHEN/THEN
**代码质量基准：** .claude/rules/02-rust-conventions.md 或 03-frontend-conventions.md
```

---

## Courier 项目特定配置

### 目录初始化

```bash
mkdir -p openspec/changes/
mkdir -p openspec/changes/archive/
touch openspec/changes/.gitkeep
touch openspec/changes/archive/.gitkeep
```

### .gitignore 补充

openspec/ 目录全部纳入版本控制，无需排除。

### change-id 命名规范

格式：`<类型>-<简短描述>`，与分支命名对齐

| 类型 | 示例 |
|---|---|
| feat | `feat-jwt-refresh-token` |
| fix | `fix-websocket-reconnect` |
| refactor | `refactor-split-handlers` |

---

## 决策记录

| 决策 | 选择 | 原因 |
|---|---|---|
| 集成方案 | C-3（brainstorming 驱动，Claude 直接写 OpenSpec 文件） | 无工具链依赖，brainstorming 对话质量高于 /opsx:propose 命令输出 |
| 文档语言 | 中文 | 与项目文档体系保持一致 |
| tasks.md 定位 | 轻量治理视图，详细步骤引用 plans/ 文件 | 避免重复维护，单一信息源 |
| 规格审查基准 | specs/ 中的 GIVEN/WHEN/THEN | 比 plans/ 文件更精准，直接对应功能验收标准 |
| 小改动路径 | 跳过 OpenSpec 文档 | 引入完整流程的成本超过收益 |

---

## 范围外事项

- 不集成 OpenSpec CLI 工具（`/opsx` 命令）
- 不修改现有 `.claude/rules/` 文件内容（仅在 CLAUDE.md 增加章节）
- 不引入新的 Cargo.toml 或 package.json 依赖
