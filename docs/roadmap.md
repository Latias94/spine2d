# Roadmap (spine2d)

本路线图以“先跑通最小闭环 + 用可移植的行为测试锁定语义”为原则，逐步扩展到完整的 Spine 4.3 功能面。

## 0. 约束与原则（已确认）

- 纯 Rust 实现，核心不依赖 C/C++、不需要 emsdk。
- 一等公民目标：`wasm32-unknown-unknown` + native。
- 对齐 Spine 导出数据：4.3.x（同时尽量保持对 4.x 较早导出的宽容解析）。
- crate 形态：`spine2d`（核心） + `spine2d-wgpu`（集成/后端），不拆太碎。

## 1. 里程碑

### M1：最小可测运行时（AnimationState 语义优先）

目标：先把“时间推进 + 事件队列 + 混合/中断/结束/释放”的行为做对，并用官方测试用例思想锁死语义。

交付：
- `spine2d::json`：最小 JSON 解析能力（仅覆盖测试所需字段）：
  - `skeleton`（版本字符串读取与兼容性检查）
  - `bones`（至少 root）
  - `events`（EventData：int/float/string/audio/volume/balance）
  - `animations.events`（EventTimeline）
- `spine2d::runtime`：
  - `SkeletonData` / `Skeleton`（最小骨架对象）
  - `Animation` / `Timeline` / `EventTimeline`
  - `AnimationStateData` / `AnimationState` / `TrackEntry`
  - 事件派发：start/interrupt/end/dispose/complete/event（顺序与时间戳对齐官方）
- 测试：
  - 先移植 `spine-csharp` 的 `AnimationStateTests` 中与事件相关的一小组 case（例如前 3~5 个），改写为 Rust 单元测试（不依赖文件，直接内嵌 JSON 字符串）。

验收：
- `cargo test -p spine2d` 通过。
- 行为测试断言：事件顺序与关键时间值匹配（允许极小浮点误差）。

### M2：核心数据结构与骨骼变换（pose 计算）

目标：实现 Skeleton pose + `update_world_transform`，为渲染输出打基础。

交付：
- JSON 解析扩展：slots/skins/attachments（先 region + mesh 的数据面），constraints（先 IK/transform/path 的数据结构，执行可后置）。
- 运行时：
  - Bone 本地/世界变换（矩阵或分解参数，选一种并固定）
  - `update_world_transform`（不含高级约束可先落地）
- 测试：
  - 解析测试：字段缺省/兼容性/错误定位
  - 变换测试：简单层级下的 world transform 数值断言

### M3：渲染无关输出层（render commands / batches）

目标：在不绑定任何图形 API 的前提下，输出可直接提交 GPU 的批次/顶点索引数据。

交付：
- `spine2d::render`：
  - 统一顶点格式（位置/UV/颜色/附加数据）
  - draw order 遍历，region/mesh 生成三角形
  - blend mode、premultiplied alpha（PMA）路径规划
  - atlas 解析与 region UV 映射（先文本 `.atlas`）
- 测试：
  - 几何输出的结构性断言（顶点数/索引数/批次数）

### M4：Animation 完整度（更多 timeline + mixing）

目标：从“只做 events”扩展到常见 timeline（rotate/translate/scale/color/attachment/deform 等），并补齐 mixing 行为。

交付：
- Timeline 覆盖面扩展（按使用频率排序）。
- 继续移植/补充官方测试思想：
  - 从 `AnimationStateTests` 移植更多 case（包含 mixing、queue、trackEnd 等边界）。

验收 / 退出标准（避免“无限加测试”）：
- 以 C++ oracle 为准：关键语义都有“哨兵场景”可回归（JSON + `.skel` 各至少一组）。
- 每个高风险轴至少 1–2 个真实资源哨兵覆盖：`MixBlend::Add`、mix-out to empty、attachment switch、deform（含 linkedmesh/weighted）、clipping、constraints（IK/transform/path）、physics（Update/Pose/Reset + jitter dt + long-run）。
- 覆盖“步长敏感”场景：至少包含 mixed dt（jitter）与 long-run（10s）两类 oracle 快照，用于捕捉 remaining/step 边界与数值漂移。
- `cargo test -p spine2d --features json,binary,upstream-smoke` 持续全绿。
- 达到以上条件后：默认不再主动扩张测试矩阵；仅在“新增特性/修复 bug/出现不一致”时新增对应最小哨兵测试。

### M5：`spine2d-wgpu` 与 demo（含 wasm）

目标：把 `spine2d` 的 render 输出接上 wgpu，提供可运行示例，验证 wasm32-unknown-unknown 路径。

交付：
- `spine2d-wgpu`：pipeline、buffers、texture bind group、批次提交。
- 示例（后续可加 `examples/` 或单独 viewer crate，暂不发布到 crates.io）。
- wasm：使用 wgpu web 后端 + 用户自行加载资源（fetch/bytes）。

进度：
- M5（进行中）：新增 `spine2d-web`（Trunk + wgpu + canvas）最小 demo，用于验证 `wasm32-unknown-unknown` 启动与渲染闭环（当前示例为程序纹理 + 内嵌 JSON/atlas）。
- M5（进行中）：`spine2d-web` 支持 `copy-dir` + `fetch(bytes)` 加载 `assets/demo.(json|atlas|png)`，验证 web 侧资源加载与贴图上传链路。
- M5（进行中）：`spine2d-web` 增加最小 DOM 控制条（Play/Pause、Restart、Fit、Speed、Animation）。
- M5（进行中）：`scripts/prepare_spine_runtimes_web_assets.py`：下载官方 `spine-runtimes` 示例导出并生成 `web_manifest.json`，让 web demo 可在本地加载真实导出资源（不提交到仓库）。

## 2. 行为参考（“以谁为准”）

我们会把“行为语义”对齐到官方维护最积极、覆盖最广的实现与测试：

- **首要行为参考**：官方的 **C# runtime 测试用例**（`spine-csharp/tests/src/AnimationStateTests.cs`）。
  - 这些测试直接描述了事件顺序、混合与边界行为，适合作为可移植规范。
- **实现细节参考**：`spine-cpp`（因为 4.3 的 `spine-c` 实际上是对 `spine-cpp` 的规则化 C API 封装）。
- **格式与概念参考**：Spine 官方文档/运行时指南（解析缺省字段、PMA、blend 等）。

说明：当不同运行时在边界细节上出现差异，优先跟随“官方测试/参考实现”一致的行为，并在 `docs/notes/` 记录差异与理由。

## 3. Rust 最佳实践约定

- **错误处理**：全部对外 API 用 `Result<T, spine2d::Error>`；解析错误提供字段路径与位置（行/列或 JSON pointer）。
- **日志**：核心 crate 默认不产生日志；如需要，使用可选 feature `tracing`（或后续决定 `log`），避免强绑日志实现。
- **异步**：核心 runtime 不做 IO、不引入 async；资源加载由上层（引擎/应用）负责，传入 `&[u8]`/`&str`。
- **unsafe**：核心 crate 维持 `#![forbid(unsafe_code)]`（当前已设置）；wgpu 集成如确有必要再评估。

## 4. 进度速记（只记录“做到了什么”）

- 复刻对齐的“准入标准”：`docs/parity.md`（done 需要有单测与/或 C++ oracle 对照信号）。
- 参考实现/语义锚点：Spine 官方 C# runtime 测试 `AnimationStateTests.cs`（来自 `spine-runtimes` 仓库）。
- 关键实现路径：
  - AnimationState：`spine2d/src/runtime/animation_state.rs`
  - 最小 JSON：`spine2d/src/json.rs`
- 测试对齐（已移植并通过）：
  - AnimationStateTests：#1（0.1 time step）、#2（1/60 time step, dispose queued）、#3（30 time step）
  - AnimationStateTests：#4（1 time step）
- 渲染对齐（新增并锁定）：
  - 修复 `RegionAttachment.updateRegion/computeWorldVertices` 的顶点顺序与 UV 映射（包含 rotate=90），用 C++ runtime 的 `SkeletonRenderer` 作为 oracle 对齐（spineboy/alien/dragon）
  - 修复 `.atlas` 多 page 解析（`dragon.atlas` 这类资源正确分配 region→page）
  - 新增工具链：`scripts/run_spine_cpp_lite_render_oracle.zsh`（支持 legacy + scenario mode）+ `spine2d/examples/render_dump.rs` + `scripts/compare_render.py`
  - AnimationStateTests：#5（interrupt）、#6（interrupt with delay）、#7（interrupt with delay and mix time）、#8（animation 0 events do not fire during mix）
  - AnimationStateTests：#9（event threshold, some animation 0 events fire during mix）、#10（event threshold, all animation 0 events fire during mix）
  - AnimationStateTests：#11（looping）、#12（not looping, track end past animation duration）
  - AnimationStateTests：#13（interrupt animation after first loop complete）、#14（add animation on empty track）、#15（end time beyond non-looping animation duration）
  - AnimationStateTests：#16（looping with animation start）、#17（looping with animation start and end）、#18（non-looping with animation start and end）、#19（mix out looping with animation start and end）
  - AnimationStateTests：#20（setAnimation with track entry mix）、#21（setAnimation twice）、#22（setAnimation twice with multiple mixing）
  - AnimationStateTests：#23（addAnimation with delay on empty track）
  - AnimationStateTests：#24（setAnimation during AnimationStateListener）
  - AnimationStateTests：#25（clearTrack）
  - AnimationStateTests：#26（setEmptyAnimation）
  - AnimationStateTests：#27（TrackEntry listener）
- 运行时实现：TrackEntry 存储从 `Rc<RefCell<_>>` 重构为 arena + generational handle（避免运行时借用 panic）。
- M2（进行中）：新增 `Skeleton::update_world_transform` 与 `Bone` 本地/世界变换数据结构，并补充基础数值测试（root/child、父级旋转影响子级平移）。
- M2（进行中）：新增 bone timeline（`rotate/translate/scale`）与 `apply_animation` 采样/应用（含最短旋转路径、loop time），并扩展 JSON `animations.bones.*` 解析与对应单元测试。
- M2（进行中）：`AnimationState::apply` 开始实际驱动 `Skeleton` pose（当前先按 `MixBlend::Replace`/`alpha=1` 应用 current track，mixing 权重后续补齐），并新增一条最小闭环集成测试。
- M2（进行中）：bone timelines 语义调整为“相对 setup pose”（rotate/translate 为偏移，scale 为倍率），并补齐 time < first frame 时的 Setup/Replace 行为，测试已覆盖非零 setup 值场景。
- M2（进行中）：`AnimationState::apply` 增加最小姿势混合（按 `mixTime/mixDuration` 对 current 与 mixingFrom 做线性 blend），并新增数值集成测试用例验证。
- M2（进行中）：支持 JSON bone timeline 的 `curve: "stepped"`（按上一帧保持）与 bezier 曲线。
- M2（进行中）：新增 `shear` bone timeline（JSON 解析 + apply），补齐骨骼 4 个基础变换（rotate/translate/scale/shear）。
- M2（进行中）：支持 JSON bone timeline 的 bezier curve（`curve: [cx1, cy1, cx2, cy2]`），采样在 **(time,value)** 空间对齐官方运行时语义（而非 0..1 百分比曲线）。
- M2→M3（进行中）：补齐 `slots/skins/attachments(region)` 的数据面（JSON 解析 + `Skeleton` slots/drawOrder），为渲染输出做准备。
- M3（起步）：新增 `spine2d::render` 的最小 `DrawList`（quad 顶点 + 索引 + draw 分段），已可从 `region` attachment 生成四边形几何（UV/颜色先占位），并有单元测试锁定输出结构。
- M3（进行中）：新增 `.atlas` 文本解析（page/region/size/xy/bounds/rotate(degrees)/orig/offset/pma/filter/repeat/scale）与 `build_draw_list_with_atlas`，支持用 atlas region 生成 UV，并按 atlas page 合批 draw。
- M3（进行中）：支持 slot `blend`（normal/additive/multiply/screen）与 atlas page `pma: true`：`Draw` 携带 `blend`/`premultiplied_alpha`，`spine2d-wgpu` 选择对应 blend state，且 PMA 下顶点颜色做预乘。
- M3（进行中）：atlas region 支持 `bounds: x,y,w,h`（官方示例常见导出格式），并与 `xy/size` 一样可驱动 UV 映射。
- M3（进行中）：atlas page 支持 `filter:` 与 `repeat:` 的解析，并在 `spine2d-wgpu` 提供从 `AtlasPage` 自动创建 `wgpu::Sampler` 的辅助函数，避免用户手写 sampler 配置。
- M3（进行中）：atlas page 支持 `scale:`（多分辨率/像素密度元信息），解析后可供上层选择合适资源版本。
- M3（进行中）：atlas region 支持 trim 元信息：`orig:`/`offset:`/`offsets:`（用于裁剪打包后的 attachment 几何对齐，region/mesh 已在渲染输出中生效）。
- M3（进行中）：atlas region `rotate:` 支持 `true/false` 与 `90/180/270`，并对齐官方 `spine-ts` 的 UV/trim 行为；新增单元测试覆盖 degrees=90 的 region quad UV+trim，以及 mesh degrees=90/180/270 + trim。
- M3（进行中）：新增 `mesh` attachment（先支持 unweighted：`vertices` 与 `uvs` 等长）解析与渲染输出（顶点/索引/atlas UV 映射），并添加单元测试覆盖。
- M3（进行中）：支持 weighted mesh（Spine JSON `vertices` 带 boneCount/weights），渲染时按多骨骼 world transform 加权求和生成顶点位置，并添加单测验证。
- M4（进行中）：支持 `animations.*.attachments.*.*.*.deform`（mesh FFD）解析与运行时 apply，写入 `Slot::deform`，并在渲染输出中对 unweighted/weighted mesh 生效；新增对应单元测试。
- M4（进行中）：支持 `animations.*.slots.*.attachment` 与 `animations.*.drawOrder` 的解析与 apply，驱动 slot attachment 切换与 draw order 重排；新增对应单元测试。
- M4（进行中）：支持 slot setup color（slots[].color）与 `animations.*.slots.*.color`（插值 RGBA + curve），并让 `DrawList` 顶点颜色随 slot 颜色变化；新增对应单元测试。
- M4（进行中）：支持 JSON `ik`（1/2 bone IK）解析与在 `Skeleton::update_world_transform` 中求解（对齐 4.3：`mix/softness/stretch/compress/uniform` + `bendPositive`），并新增数值测试验证末端逼近 target。
- M4（进行中）：支持 `animations.*.ik`（IK timeline）解析与 apply（`mix/softness` 插值 + `bendPositive`），并新增测试验证 mix=0 禁用约束、mix/softness 插值生效。
- M4（进行中）：曲线（`curve`）语义对齐官方：Bezier 曲线在 **(time,value)** 空间，且多值 timeline（如 translate/scale/shear、color、ik、path mix、transform mix）为每个 valueIndex 单独存储与采样 curve；已通过 C++ oracle 对比 `spineboy-pro.json` 的 `run` 动画多个时间点，pose diff（eps=1e-3）为 0。
- M4（进行中）：支持 JSON `transform` constraint（absolute/relative × local/world）解析与在 `Skeleton::update_world_transform` 中按 `order` 与 IK 混排执行；支持 `animations.*.transform` mix timeline；新增单元测试锁定四分支旋转、absolute-world 平移、timeline 插值与跨类型 order。
- M4（进行中）：支持 JSON `path` constraint 数据结构与解析（含 4.3 `skins` 数组格式、`type: "path"` attachment）；支持 `animations.*.path` 的 position/spacing/mix timelines 驱动运行时参数；已实现 path constraint 求解并新增最小数值测试（constantSpeed true/false、mix=0 禁用）。
- M4（进行中）：PathConstraint 求解测试覆盖面扩展：`positionMode=percent`、`spacingMode=percent/proportional/length`、`rotateMode=chain/chainScale`（含 2 bone chain 与 scale 校验）、`closed=true`（position wrap）、`mixRotate<1`（渐进旋转）、chain+spacing=0（走 `positions[p+2]` 切线角分支）。
- M4（进行中）：新增真实导出数据 `vine-pro.json` smoke test：解析成功 + `Skeleton::update_world_transform()` 不 panic，且骨骼矩阵/位移均为有限数值（非 NaN/Inf）。
- JSON 解析：新增 `SkeletonData::from_json_str_with_scale(input, scale)`（默认 `from_json_str` 等价于 `scale=1`），并补测试锁定 PathConstraint 的“按 mode 条件缩放”行为。
- 测试：新增多份官方 `examples/*/export/*.json` 的 smoke tests（解析 + apply 一条动画 + `update_world_transform` + `build_draw_list`，并检查无 NaN/Inf），用于快速回归真实数据兼容性。
- M4（进行中）：启动 Binary `.skel` loader（feature `binary`），覆盖 bones/slots/constraints/skins/animations 的解析（含 4.x 细分 timeline 类型，如 `translateX/Y`、`scaleX/Y`、`shearX/Y`、`inherit`、slot `rgb/alpha`），并新增 `.skel` smoke test（`spineboy-pro.skel` 可解析与采样一帧）。
- 修正：对齐 `spine-cpp` 的二进制默认值编码：`IkConstraintData.mix` 默认 `0`（flags 缺省不赋值），`TransformConstraintData` 的各 `mix*` 默认 `0`（仅在 flag 置位时读取），避免 `.skel` 加载后约束默认误开启导致 pose 偏移。
- 测试：扩展 C++ oracle 支持 `.skel` 输入，并为多份官方 examples（spineboy/tank/dragon/mix-and-match/goblins）补齐 `.skel` 场景快照，作为行为回归信号（与 C++ oracle 输出对齐，eps=1e-3）。
- 测试：新增 `spineboy-pro.json` 的 pose 回归快照（与 C++ oracle 输出对齐，eps=1e-3，`--features upstream-smoke`）。
- 测试：扩展 pose parity 到“场景级”对齐（C++ oracle scenario）：`run -> walk`（mix=0.2，dt=0.1/0.25）与 multi-track `run + aim`（dt=0.2）。
- 测试：补齐 `spineboy-pro.json run -> walk (mix=0.2)` 的场景快照：`t=0.4`（切换后 dt=0.1）与 `t=0.55`（切换后 dt=0.25），作为单轨道混合过渡的 pose oracle 回归信号。
- 测试：扩展 TrackEntry `additive` 对齐覆盖：新增 `run + aim(additive, alpha=0.5) t=0.2` 与 `aim(additive) -> shoot(additive) t=0.4` 的场景快照，锁定 `alpha` 与 additive 在混合路径中的官方语义。
- 测试：补齐 TrackEntry additive/replace 在 mixingFrom/out 时的 blend 选择逻辑：新增 spineboy `aim(additive) -> shoot(replace)` 与 `aim(replace) -> shoot(additive)` 的场景快照，锁定 `applyMixingFrom` 使用 per-timeline `MixFrom` 与 additive 标志的分支选择。
- 测试：补齐多段 mixing 链中的 `holdMix` 对齐覆盖：新增 spineboy/tank 多段场景快照，锁定 `computeHold/holdMix` 在真实资源上的行为。
- 工具：C++ oracle 场景模式新增 TrackEntry threshold 参数（`--entry-alpha-attachment-threshold`/`--entry-mix-attachment-threshold`/`--entry-mix-draw-order-threshold`/`--entry-event-threshold`），便于锁定 thresholds 的边界语义。
- 测试：新增 spineboy thresholds 场景快照：`shoot(alpha=0.5, alphaAttachmentThreshold=0.6) t=0.1`（验证 attachment gate）与 `shoot(mixAttachmentThreshold=0, mixDrawOrderThreshold=0) -> empty(mix=0.2) t=0.2`（验证 thresholds + unkeyed 交互）。
- 测试：扩展 `MixBlend::Add` 的“跨模块”叠加覆盖：新增 `tank-pro.json drive(track0) + shoot(track1:add) t=0.4` 的场景快照，覆盖 path constraint + drawOrder/slots/attachments + clipping worldVertices 的叠加语义。
- 测试：补齐 `MixBlend::Add` 在 mixingFrom/out 分支的对齐覆盖：新增 `tank-pro.json drive(track0) + shoot(track1:add) -> empty(mix=0.2) t=0.35` 的场景快照，锁定 Add 轨道淡出时 attachments/drawOrder/clipping/constraints 的官方行为。
- 测试：补齐 `mixDrawOrderThreshold` 的可观测边界对齐：新增 `tank-pro.json shoot(track1) -> shoot(track1) (mix=0.2) t=0.4` 场景快照，分别覆盖 `mixDrawOrderThreshold=0`（不应用 drawOrder）与 `mixDrawOrderThreshold=1`（强制应用 drawOrder）。
- 测试：移植 `spine-c-unit-tests` 的 headless smoke（spineboy/raptor/goblins：顺序播放所有动画直到结束，不崩溃），并补齐 spineboy 官方 headless `run -> walk` 过渡的多时间点采样回归（`t=0.25/0.333333/0.416667/0.5`）。
- 工具：C++ oracle 与 Rust pose dump 扩展输出 slots/drawOrder/constraints，并由 `scripts/compare_pose.py` 支持对齐 diff（便于锁定 attachment/constraint 等非骨骼状态差异）。
- 修正：`applyMixingFrom` 在 TrackEntry `additive=true` 且 `MixDirection::Out` 时，Attachment/DrawOrder 由 per-timeline `MixFrom` 与 threshold 共同决定（对齐 spine-cpp），并由场景对齐锁定：track1 `aim(additive) -> shoot(additive)`（crosshair/muzzle-glow）。
- 增补：`AnimationState::add_empty_animation`（匹配上游 AddEmptyAnimation 的 delay 调整语义），并在 `pose_dump_scenario` 支持 `--add-empty` 方便做更多场景对齐。
- 修正：旋转混合（rotation accumulator）对齐 `spine-cpp` 的 `sign(0)=0` 语义，避免首帧混合在负向 diff 下错误额外叠加 360°（导致 180° 翻转），并由 scenario parity 锁定回归。
- M4（进行中）：补齐 TrackEntry 行为开关：`reverse`（倒放采样，禁用 event timeline 派发）与 `shortestRotation`（禁用 rotation accumulator），并新增单元测试锁定语义。
- JSON 兼容性：支持 `type: "linkedmesh"`（解析后解析：从 parent mesh 复制 vertices/uvs/triangles），并将更多官方 example（如 goblins/chibi-stickers/mix-and-match）纳入 smoke tests。
- JSON 兼容性：支持 `type: "boundingbox"` 与 `type: "clipping"` 的解析；`spine2d::render` 已支持 `clipping` 对 `region/mesh` 做几何裁剪（并新增单元测试），`boundingbox` 仍仅解析不输出；coin/spineboy/tank 等依赖这些类型的官方 example 已纳入 smoke tests。
- 测试：移植 `spine-c-unit-tests` 的 `MemoryTestFixture` 中几何用例（`Triangulator` 与 `SkeletonClipping::clipTriangles`），为“裁剪/三角剖分”提供上游回归信号。
- M4（进行中）：支持 `sequence`（region/mesh）与 `SequenceTimeline`（含 `MixDirection::Out` 的 reset 语义），渲染根据 `Slot.sequence_index` 选择实际帧贴图路径；新增单元测试锁定行为。
- 测试：新增 `dragon-ess.json flying t=0.25` 的 C++ oracle 快照（sequenceIndex=3 的对齐信号），用于锁定 SequenceTimeline 在真实示例上的行为。
- 测试：新增 `dragon-ess.json flying t=0.65/0.76/0.85/0.98` 的 C++ oracle 快照，用于锁定 SequenceTimeline 在关键帧 0.6/0.7333/0.8/0.9667 前后 frame index 的变化语义。
- 测试：新增 `dragon-ess.json flying -> empty (mix=0.2) t=0.35` 的 C++ oracle 快照，用于锁定 SequenceTimeline 在 mix out 时将 slot `sequenceIndex` reset 回 `-1` 的语义。
- JSON 兼容性：rotate timeline 支持 `value`/`angle` 两种字段名（便于直接解析官方示例导出）。
- 修正：RotateTimeline 对齐 `spine-cpp`：纯 apply（alpha==1）走相对值公式（不额外做 `wrapDegrees`）；混合路径仍由 rotation accumulator 负责短路径与跨帧方向检测。
- 修正：`AnimationState::apply` 的旋转混合引入 `spine-cpp` 风格的 per-entry rotation accumulator（跨帧方向检测），进一步贴近官方旋转混合语义。
- M4（进行中）：`AnimationState::apply` 进入 `spine-cpp` 风格的 per-timeline 混合：实现 `computeHold`（Current/Setup/First + Hold/HoldMix）、TrackEntry `additive`、`unkeyedState`，并补充 attachment/drawOrder thresholds 的单测锁语义。
- M4（完成）：对齐 `spine-cpp` 的 propertyId：使用 `Property<<32 | index` 编码，并为 VertexAttachment/Sequence 分配进程级递增 ID（Deform/Sequence 使用 `slotIndex<<16 | id`）。
- `spine2d-wgpu`：`examples/basic.rs` 确保首帧触发 `RedrawRequested`（避免窗口空白）。
- M4（进行中）：支持 two-color tint（slot setup `dark` + `rgba2`/`rgb2` timelines），并在 `spine2d-wgpu` shader 端打通 darkColor；新增 `tank-pro.json shoot t=0.3` oracle 回归测试锁定语义。
- M4（进行中）：支持 `point` attachment（解析 + world position/rotation 计算辅助），用于生成发射点/挂点等玩法数据；新增单元测试锁定基础语义。
- M4（进行中）：补齐 skins/active 语义：解析 `skinRequired` 与 per-skin bones/constraints 列表；`Skeleton` 初始无 skin；`update_cache` 计算 bone/constraint active；timeline apply 对 inactive 做 gate；并新增 mix-and-match `accessories/backpack` 的上游 smoke 测试锁定关键行为。
- 修正：PathConstraint 应用后只重算“非约束骨骼”的 descendants，避免把同一条链上的骨骼当作 descendants 重新计算从而覆盖约束结果；该问题会导致 mix-and-match 的 leg/arm path chain world transform 严重偏离官方 C++ runtime，已通过 oracle scenario 对齐锁定（eps=1e-3）。
- 优化：约束应用阶段消除每帧临时分配（预计算 bone children；复用 descendants update mask / stack / excluded scratch），避免长时间运行形成 alloc 热点，便于 wasm/移动端。
- 修正：DeformTimeline/Slot deform 对齐 `spine-cpp` 的 `timelineAttachment` 语义（linkedmesh 按父 mesh 作为 deform 目标；attachment 切换仅在 timelineAttachment 改变时清空 deform），并同步修正 linkedmesh 默认父 skin 为 `default`。
- 修正：渲染 clipping 对齐 `spine-cpp`：clipping attachment 触发 `clipStart` 后不调用 `clipEnd(slot)`，使 `end` 语义与官方一致（包含 `end` 指向自身时剪裁持续到 render 结束）；已由 `coin-pro.json` render oracle 对齐锁定。
- JSON 兼容性：支持骨骼轴向 timeline `translatex/translatey`（Spine 4.3 导出格式），并新增 `sack_walk_physics_none_t0_5.json` oracle 快照锁定行为。
- M4（进行中）：支持 Physics Constraints（解析 + runtime update/apply），并新增 `cloud_pot_playing_in_the_rain_physics_t0_5.json` / `sack_walk_physics_t0_5.json` oracle 快照锁定行为（含 physicsConstraints 运行时状态）。
- M4（进行中）：Physics oracle 覆盖面扩展：新增 `celestial_circus_wind_idle_physics_t0_5.json` / `snowglobe_idle_physics_t0_5.json`（含 `.skel` 版本）快照锁定行为。
- M4（进行中）：Physics 模式语义覆盖：新增 Update→Pose 与 Update→Reset→Update 的场景快照（含 `.skel` 版本），锁定 mode 切换时的 `lastTime/remaining/offsets` 等状态对齐。
- M4（进行中）：Physics 模式语义覆盖（snowglobe）：新增 `snowglobe_idle_physics_update_reset_update_t1_0.json`（含 `.skel` 版本）快照锁定 Update→Reset→Update roundtrip。
- M4（进行中）：Physics 时间推进覆盖：新增 long-run（10s）与 jitter dt（混合步长）场景快照（含 `.skel` 版本），用于捕捉数值漂移与 step/remaining 边界差异。
- M4（进行中）：Physics 时间推进覆盖（snowglobe）：新增 `snowglobe_idle_physics_t10_0.json`（含 `.skel` 版本）oracle 快照，锁定 10s long-run 下 physicsConstraints 状态与骨骼世界变换漂移。
- M4（进行中）：Physics 时间推进覆盖（snowglobe）：新增 `snowglobe_idle_physics_jitter_dt_t10_0.json`（含 `.skel` 版本）oracle 快照，锁定 mixed dt long-run 下 remaining/step 边界与漂移。
- M4（进行中）：Physics 时间推进覆盖（cloud-pot）：新增 `cloud_pot_playing_in_the_rain_physics_jitter_dt_t10_0.json`（含 `.skel` 版本）oracle 快照，补齐另一个资源的 mixed dt long-run 哨兵。
- M4（完成）：Physics “哨兵覆盖”已满足退出标准（Update/Pose/Reset + jitter dt + 10s long-run，且含 `.skel` 版本）。后续仅在发现行为差异时追加最小回归场景。
- M4（进行中）：Physics “压力测试”覆盖：以 `snowglobe`（physics constraints 数量最多）新增 jitter dt 与 Update→Pose→Update roundtrip 的场景快照（含 `.skel` 版本），用于捕捉 mode 切换与步长抖动导致的状态差异。
- M4（进行中）：Physics 与 AnimationState 交互：新增 `sack walk -> hello (mix=0.2)` 的 physics 场景快照（含 `.skel` 版本），锁定“混合期间 + 物理更新”的叠加语义。
- M4（进行中）：Physics 与多轨叠加：新增 `sack walk(track0) + hello(track1, MixBlend::Add)` 的 physics 场景快照（含 `.skel` 版本），覆盖多 track + Add 叠加在 physics 下的语义。
- M4（进行中）：Physics 与叠加层生命周期：新增 `sack track1(hello, MixBlend::Add) -> empty(mix=0.2)` 的 physics 场景快照（含 `.skel` 版本），覆盖 Add 轨道淡出与物理更新的叠加语义。
- M4（进行中）：Physics “最大压力”版本：在 `snowglobe` 上新增 `snowglobe_idle_plus_shake_add_to_empty_mix0_2_physics_t0_6.json`，并补齐 mixed dt（jitter）版本 `snowglobe_idle_plus_shake_add_to_empty_mix0_2_physics_jitter_dt_t0_6.json`（均含 `.skel` 版本），用于覆盖大量 physics constraints 下的叠加层淡出语义与步长边界。
- M4（进行中）：多轨叠加补齐：新增 `spineboy_run_plus_portal_add_to_empty_mix0_2_t0_6.json`（含 mixed dt 与 `.skel` 版本）oracle 快照，锁定 `shear/scale/translate` 在 `MixBlend::Add` 下的叠加与 mix-out 语义。
- M4（进行中）：多轨叠加补齐：新增 `spineboy_run_plus_shoot_add_to_empty_mix0_2_t0_6.json`（含 mixed dt 与 `.skel` 版本）oracle 快照，锁定 slot `rgba` 在 `MixBlend::Add` 下的叠加与 mix-out 语义。
- M4（进行中）：多轨叠加补齐：新增 `spineboy_run_plus_shoot_add_alpha0_5_t0_4.json`（含 mixed dt 与 `.skel` 版本）oracle 快照，锁定 `MixBlend::Add` 下 entry `alpha` 对 slot RGBA/attachment 的缩放语义。
- M4（进行中）：two-color tint 对齐补齐：新增 `tank_drive_plus_shoot_add_alpha0_5_t0_4.json` / `tank_drive_plus_shoot_add_alpha0_5_to_empty_t0_35.json`（均含 mixed dt 与 `.skel` 版本）oracle 快照，锁定 `MixBlend::Add` + `alpha` + mix-out 下 `rgba2/rgb2` 的缩放语义。
- M4（进行中）：drawOrder 混合语义补齐：新增 `tank_drive_plus_shoot_add_to_empty_mixDrawOrderThreshold_0_t0_55.json`/`tank_drive_plus_shoot_add_to_empty_mixDrawOrderThreshold_1_t0_55.json`（含 mixed dt 与 `.skel` 版本）oracle 快照，锁定 `MixBlend::Add` 轨道在 mixingFrom/out 下的 drawOrder 应用阈值语义。
- M4（进行中）：DeformTimeline 场景补齐：新增 `tank_drive_plus_shoot_add_smoke_glow_deform_t0_25.json` 与 `tank_drive_plus_shoot_add_to_empty_smoke_glow_deform_t0_35.json`（含 mixed dt 与 `.skel` 版本）oracle 快照，锁定 `attachments.*.deform` 在 Add 叠加与 mix-out 下的 worldVertices 行为。
- M4（进行中）：linkedmesh deform 场景补齐：新增 `goblins_walk_skin_goblingirl_left_foot_deform_t0_3.json`（含 mixed dt 与 `.skel` 版本）oracle 快照，锁定 linkedmesh 继承父 mesh deform 的 `timelineAttachment` 行为。
- M4（进行中）：weighted mesh deform 场景补齐：新增 `hero_idle_head_deform_t0_55.json` 与 `hero_idle_plus_run_add_head_deform_t0_55.json`（均含 mixed dt 与 `.skel` 版本）oracle 快照，锁定 weighted deform 在单轨与 `MixBlend::Add` / mix-out 下的 worldVertices 行为。
- M4（进行中）：owl 多 deform timeline 场景补齐：新增 `owl_up_head_base_deform_t0_55.json` 与 `owl_up_plus_left_add_head_base_deform_t0_55.json`（均含 mixed dt 与 `.skel` 版本）oracle 快照，锁定“同一动画包含多条 deform timeline”与 multi-track `MixBlend::Add` 的叠加语义。
- M4（进行中）：owl 小顶点 deform 场景补齐：新增 `owl_up_l_wing_deform_t0_55.json` 与 `owl_up_plus_left_add_l_wing_deform_t0_55.json`（均含 mixed dt 与 `.skel` 版本）oracle 快照，用更小 worldVertices 目标提高 deform 混合差异的敏感度。
- M4（进行中）：owl 小顶点 deform 场景补齐：新增 `owl_up_r_wing_deform_t0_55.json` 与 `owl_up_plus_left_add_r_wing_deform_t0_55.json`（均含 mixed dt 与 `.skel` 版本）oracle 快照，覆盖左右翼对称目标。
- M4（进行中）：owl attachment timeline 切换补齐：新增 `owl_up_plus_blink_l_wing_t0_5.json` 与 `owl_up_plus_blink_to_empty_mix0_2_l_wing_t0_55.json`（均含 mixed dt 与 `.skel` 版本）oracle 快照，锁定 blink 触发的 attachment 切换与 track1 淡出期间的语义。
- 已知待对齐（高优先级）：继续扩展“oracle 锁定”的覆盖面（更多动画/时间点/场景），逐步把 `compare_pose.py --eps 1e-4` 下的残余浮点差异也收敛掉（目前 `tank-pro.json drive t=0.3` 在 eps=1e-3 下已对齐）。
