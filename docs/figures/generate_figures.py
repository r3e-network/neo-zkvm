#!/usr/bin/env python3
"""Generate the Neo zkVM figure set.

The SVGs are documentation assets, but they are generated from one source so the
English and Chinese versions stay structurally identical.
"""

from __future__ import annotations

from dataclasses import dataclass
from html import escape
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"
EN_DIR = DOCS / "figures"
ZH_DIR = DOCS / "zh" / "figures"


@dataclass(frozen=True)
class Tone:
    stroke: str
    fill: str
    text: str = "#0f172a"


TONES = {
    "slate": Tone("#475569", "#f8fafc"),
    "green": Tone("#16a34a", "#ecfdf3"),
    "blue": Tone("#2563eb", "#eff6ff"),
    "amber": Tone("#d97706", "#fffbeb"),
    "purple": Tone("#7c3aed", "#f5f3ff"),
    "red": Tone("#dc2626", "#fef2f2"),
    "teal": Tone("#0f766e", "#f0fdfa"),
}

ARROWS = {
    "slate": "#475569",
    "green": "#16a34a",
    "blue": "#2563eb",
    "amber": "#d97706",
    "purple": "#7c3aed",
    "red": "#dc2626",
    "teal": "#0f766e",
}


class Svg:
    def __init__(self, width: int, height: int, title: str, subtitle: str) -> None:
        self.width = width
        self.height = height
        self.parts: list[str] = []
        self.parts.append(
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" '
            f'height="{height}" viewBox="0 0 {width} {height}" role="img" '
            f'aria-labelledby="title desc">'
        )
        self.parts.append("<title id=\"title\">" + escape(title) + "</title>")
        self.parts.append("<desc id=\"desc\">" + escape(subtitle) + "</desc>")
        self.parts.append(
            """
<defs>
  <filter id="shadow" x="-8%" y="-8%" width="116%" height="124%">
    <feDropShadow dx="0" dy="8" stdDeviation="10" flood-color="#0f172a" flood-opacity="0.10"/>
  </filter>
  <marker id="arrow-slate" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
    <path d="M0,0 L0,6 L9,3 z" fill="#475569"/>
  </marker>
  <marker id="arrow-green" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
    <path d="M0,0 L0,6 L9,3 z" fill="#16a34a"/>
  </marker>
  <marker id="arrow-blue" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
    <path d="M0,0 L0,6 L9,3 z" fill="#2563eb"/>
  </marker>
  <marker id="arrow-amber" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
    <path d="M0,0 L0,6 L9,3 z" fill="#d97706"/>
  </marker>
  <marker id="arrow-purple" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
    <path d="M0,0 L0,6 L9,3 z" fill="#7c3aed"/>
  </marker>
  <marker id="arrow-red" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
    <path d="M0,0 L0,6 L9,3 z" fill="#dc2626"/>
  </marker>
  <marker id="arrow-teal" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
    <path d="M0,0 L0,6 L9,3 z" fill="#0f766e"/>
  </marker>
  <style>
    .bg { fill: #f8fafc; }
    .title { font: 700 30px "Segoe UI", "Microsoft YaHei", Arial, sans-serif; fill: #0f172a; }
    .subtitle { font: 400 16px "Segoe UI", "Microsoft YaHei", Arial, sans-serif; fill: #475569; }
    .section { font: 700 14px "Segoe UI", "Microsoft YaHei", Arial, sans-serif; fill: #64748b; letter-spacing: 0.08em; }
    .box-title { font: 700 19px "Segoe UI", "Microsoft YaHei", Arial, sans-serif; fill: #0f172a; }
    .box-subtitle { font: 500 13px "Segoe UI", "Microsoft YaHei", Arial, sans-serif; fill: #475569; }
    .item { font: 400 13px "Segoe UI", "Microsoft YaHei", Arial, sans-serif; fill: #334155; }
    .small { font: 400 12px "Segoe UI", "Microsoft YaHei", Arial, sans-serif; fill: #64748b; }
    .label { font: 600 12px "Segoe UI", "Microsoft YaHei", Arial, sans-serif; fill: #334155; }
    .formula { font: 500 16px "Cascadia Code", Consolas, monospace; fill: #111827; }
    .mono { font: 500 13px "Cascadia Code", Consolas, monospace; fill: #334155; }
  </style>
</defs>
"""
        )
        self.parts.append(f'<rect class="bg" x="0" y="0" width="{width}" height="{height}"/>')
        self.text(48, 56, title, "title")
        self.text(48, 84, subtitle, "subtitle")

    def text(self, x: float, y: float, value: str, cls: str = "item", anchor: str = "start") -> None:
        self.parts.append(
            f'<text x="{x:.1f}" y="{y:.1f}" class="{cls}" text-anchor="{anchor}">'
            + escape(value)
            + "</text>"
        )

    def line_text(self, x: float, y: float, lines: list[str], cls: str = "item", line_gap: int = 20) -> None:
        for idx, line in enumerate(lines):
            self.text(x, y + idx * line_gap, line, cls)

    def box(
        self,
        x: float,
        y: float,
        w: float,
        h: float,
        title: str,
        subtitle: str = "",
        items: list[str] | None = None,
        tone: str = "slate",
        section: str | None = None,
    ) -> None:
        t = TONES[tone]
        self.parts.append(
            f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="8" fill="{t.fill}" '
            f'stroke="{t.stroke}" stroke-width="2" filter="url(#shadow)"/>'
        )
        if section:
            self.text(x + 18, y + 26, section.upper(), "section")
            title_y = y + 54
        else:
            title_y = y + 32
        self.text(x + 18, title_y, title, "box-title")
        cursor = title_y + 23
        if subtitle:
            self.text(x + 18, cursor, subtitle, "box-subtitle")
            cursor += 26
        for item in items or []:
            self.parts.append(
                f'<circle cx="{x + 25}" cy="{cursor - 5}" r="4" fill="none" '
                f'stroke="{t.stroke}" stroke-width="1.8"/>'
            )
            self.text(x + 38, cursor, item, "item")
            cursor += 23

    def note(self, x: float, y: float, w: float, h: float, title: str, lines: list[str], tone: str = "amber") -> None:
        self.box(x, y, w, h, title, "", [], tone=tone)
        self.line_text(x + 18, y + 59, lines, "item", 20)

    def arrow(
        self,
        x1: float,
        y1: float,
        x2: float,
        y2: float,
        label: str | None = None,
        tone: str = "slate",
        dashed: bool = False,
    ) -> None:
        dash = ' stroke-dasharray="7 6"' if dashed else ""
        color = ARROWS[tone]
        self.parts.append(
            f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{color}" '
            f'stroke-width="2.2"{dash} marker-end="url(#arrow-{tone})"/>'
        )
        if label:
            self.text((x1 + x2) / 2, (y1 + y2) / 2 - 8, label, "label", "middle")

    def poly_arrow(
        self,
        points: list[tuple[float, float]],
        label: str | None = None,
        tone: str = "slate",
        dashed: bool = False,
    ) -> None:
        dash = ' stroke-dasharray="7 6"' if dashed else ""
        color = ARROWS[tone]
        path_points = " ".join(f"{x:.1f},{y:.1f}" for x, y in points)
        self.parts.append(
            f'<polyline points="{path_points}" fill="none" stroke="{color}" stroke-width="2.2"{dash} '
            f'marker-end="url(#arrow-{tone})"/>'
        )
        if label and len(points) >= 2:
            mid = points[len(points) // 2]
            self.text(mid[0], mid[1] - 8, label, "label", "middle")

    def decision(self, cx: float, cy: float, w: float, h: float, title: str, tone: str = "amber") -> None:
        t = TONES[tone]
        points = [
            (cx, cy - h / 2),
            (cx + w / 2, cy),
            (cx, cy + h / 2),
            (cx - w / 2, cy),
        ]
        path_points = " ".join(f"{x:.1f},{y:.1f}" for x, y in points)
        self.parts.append(
            f'<polygon points="{path_points}" fill="{t.fill}" stroke="{t.stroke}" '
            f'stroke-width="2" filter="url(#shadow)"/>'
        )
        self.text(cx, cy + 5, title, "label", "middle")

    def formula_box(self, x: float, y: float, w: float, h: float, title: str, formulas: list[str], tone: str = "blue") -> None:
        self.box(x, y, w, h, title, "", [], tone=tone)
        for idx, formula in enumerate(formulas):
            self.text(x + 22, y + 62 + idx * 32, formula, "formula")

    def finish(self) -> str:
        self.parts.append("</svg>\n")
        return "\n".join(self.parts)


def architecture(lang: str) -> str:
    en = lang == "en"
    title = "Neo zkVM Architecture" if en else "Neo zkVM 架构"
    subtitle = (
        "Production proof tooling around one canonical Neo VM execution core."
        if en
        else "围绕唯一规范 Neo VM 执行核心构建的生产级证明工具链。"
    )
    s = Svg(1440, 900, title, subtitle)
    s.box(
        56,
        150,
        270,
        230,
        "Developer + CLI" if en else "开发者与 CLI",
        "neo-zkvm-cli",
        ["run / prove", "asm / disasm / debug", "inspect scripts and traces"]
        if en
        else ["run / prove", "asm / disasm / debug", "脚本检查与执行轨迹"],
        "blue",
        "entry" if en else "入口",
    )
    s.box(
        400,
        115,
        300,
        245,
        "Proof Orchestrator" if en else "证明编排器",
        "neo-zkvm-prover",
        ["selects Execute / Mock / SP1", "caps gas with max_cycles", "builds PublicInputs"]
        if en
        else ["选择 Execute / Mock / SP1", "用 max_cycles 限制 gas", "构造 PublicInputs"],
        "green",
        "host" if en else "主机侧",
    )
    s.box(
        400,
        420,
        300,
        220,
        "Guest Boundary" if en else "Guest 边界",
        "neo-vm-guest",
        ["ProofInput / ProofOutput ABI", "canonical bincode encoding", "deterministic zk syscalls"]
        if en
        else ["ProofInput / ProofOutput ABI", "规范 bincode 编码", "确定性 zk syscall"],
        "amber",
        "shared abi" if en else "共享 ABI",
    )
    s.box(
        400,
        705,
        300,
        135,
        "Shared Execution Core" if en else "共享执行核心",
        "neo-vm-rs",
        ["canonical StackValue", "opcode semantics", "interpreter callbacks"]
        if en
        else ["规范 StackValue", "Opcode 语义", "解释器回调"],
        "purple",
    )
    s.box(
        790,
        115,
        290,
        245,
        "SP1 Proving Stack" if en else "SP1 证明栈",
        "SP1 SDK + guest ELF",
        ["compressed proof", "PLONK proof", "Groth16 proof"]
        if en
        else ["压缩证明", "PLONK 证明", "Groth16 证明"],
        "red",
        "crypto" if en else "密码学",
    )
    s.box(
        790,
        420,
        290,
        220,
        "Proof Artifact" if en else "证明产物",
        "NeoProof",
        ["ProofOutput", "proof bytes", "PublicInputs + vkey hash + mode"]
        if en
        else ["ProofOutput", "证明字节", "PublicInputs + vkey hash + mode"],
        "teal",
    )
    s.box(
        1135,
        350,
        250,
        290,
        "Verifier Policy" if en else "验证器策略",
        "neo-zkvm-verifier",
        ["format version", "output/public input binding", "SP1 or mock checks", "fail closed"]
        if en
        else ["格式版本", "输出与公开输入绑定", "SP1 或 mock 校验", "失败即拒绝"],
        "green",
        "verify" if en else "验证",
    )
    s.note(
        790,
        705,
        595,
        115,
        "Design invariant" if en else "设计不变量",
        [
            "Neo zkVM does not carry a second NeoVM implementation."
            if en
            else "Neo zkVM 不维护第二套 NeoVM 实现。",
            "Execution semantics live in neo-vm-rs and are wrapped for proofs."
            if en
            else "执行语义属于 neo-vm-rs，zkVM 只负责证明封装。",
        ],
        "amber",
    )
    s.arrow(326, 265, 400, 238, "prove", "blue")
    s.arrow(326, 292, 400, 515, "run", "blue")
    s.arrow(550, 360, 550, 420, "ProofInput", "green")
    s.arrow(550, 640, 550, 705, "execute", "amber")
    s.arrow(700, 238, 790, 238, "SP1 mode", "red")
    s.arrow(700, 515, 790, 515, "output", "teal")
    s.arrow(935, 360, 935, 420, "proof bytes", "red")
    s.arrow(1080, 530, 1135, 500, "verify", "green")
    s.poly_arrow([(1260, 350), (1260, 238), (1080, 238)], "vkey / proof", "red", True)
    return s.finish()


def dataflow(lang: str) -> str:
    en = lang == "en"
    s = Svg(
        1640,
        930,
        "Neo zkVM Dataflow" if en else "Neo zkVM 数据流",
        "How bytes become execution output, public inputs, proof bytes, and verification evidence."
        if en
        else "脚本字节如何变成执行输出、公开输入、证明字节与验证证据。",
    )
    y = 160
    boxes = [
        (
            55,
            y,
            "Source Request" if en else "源请求",
            "script + arguments + gas",
            ["CLI", "library API", "test harness"] if en else ["CLI", "库 API", "测试框架"],
            "blue",
        ),
        (
            315,
            y,
            "ProofInput" if en else "ProofInput",
            "serialized with bincode",
            ["script: Vec<u8>", "arguments: Vec<StackItem>", "gas_limit: u64"],
            "amber",
        ),
        (
            595,
            y,
            "Host Execution" if en else "主机侧执行",
            "neo-vm-guest -> neo-vm-rs",
            ["validate input", "estimate/cap gas", "interpret with zk syscalls"] if en else ["校验输入", "估算并限制 gas", "使用 zk syscall 解释执行"],
            "green",
        ),
        (
            875,
            y,
            "ProofOutput" if en else "ProofOutput",
            "deterministic VM result",
            ["state", "result", "gas_consumed", "error"],
            "teal",
        ),
        (
            1155,
            y,
            "PublicInputs" if en else "PublicInputs",
            "verification binding",
            ["script_hash", "input_hash", "output_hash", "gas + success"] if en else ["script_hash", "input_hash", "output_hash", "gas + success"],
            "purple",
        ),
        (
            1395,
            y,
            "NeoProof" if en else "NeoProof",
            "portable proof envelope",
            ["output", "proof_bytes", "public_inputs", "vkey_hash + mode"] if en else ["output", "proof_bytes", "public_inputs", "vkey_hash + mode"],
            "red",
        ),
    ]
    for x, yy, title, subtitle, items, tone in boxes:
        s.box(x, yy, 205 if x < 1395 else 200, 235, title, subtitle, items, tone)
    arrow_points = [(260, 277, 315, 277), (520, 277, 595, 277), (800, 277, 875, 277), (1080, 277, 1155, 277), (1360, 277, 1395, 277)]
    for x1, y1, x2, y2 in arrow_points:
        s.arrow(x1, y1, x2, y2, None, "slate")

    s.box(
        595,
        505,
        485,
        175,
        "SP1 Guest Re-execution" if en else "SP1 Guest 重新执行",
        "neo-zkvm-program",
        ["reads ProofInput inside zkVM", "executes the same guest boundary", "commits PublicInputs as public values"]
        if en
        else ["在 zkVM 内读取 ProofInput", "执行同一个 guest 边界", "把 PublicInputs 作为公开值提交"],
        "red",
        "cryptographic path" if en else "密码学路径",
    )
    s.box(
        1155,
        505,
        440,
        175,
        "Verifier Evidence" if en else "验证证据",
        "host checks before accepting",
        ["proof mode policy", "proof bytes / vkey", "public values equal PublicInputs"]
        if en
        else ["证明模式策略", "证明字节 / vkey", "公开值等于 PublicInputs"],
        "green",
        "verification" if en else "验证",
    )
    s.arrow(697, 395, 697, 505, "input", "red")
    s.arrow(980, 505, 1155, 592, "committed public values", "red")
    s.arrow(1495, 395, 1375, 505, "proof envelope", "green")
    s.formula_box(
        55,
        735,
        1540,
        130,
        "Hash and commitment binding" if en else "哈希与承诺绑定",
        [
            "H_script = SHA256(script)",
            "H_input = SHA256(Encode(ProofInput)); H_output = SHA256(Encode(ProofOutput))",
            "commitment = SHA256(H_script || H_input || H_output || LE64(gas_consumed) || success_byte)",
        ]
        if en
        else [
            "H_script = SHA256(script)",
            "H_input = SHA256(Encode(ProofInput)); H_output = SHA256(Encode(ProofOutput))",
            "commitment = SHA256(H_script || H_input || H_output || LE64(gas_consumed) || success_byte)",
        ],
        "blue",
    )
    return s.finish()


def workflow(lang: str) -> str:
    en = lang == "en"
    s = Svg(
        1500,
        980,
        "Neo zkVM Workflow" if en else "Neo zkVM 工作流",
        "Operational paths for local execution, proof generation, and verification."
        if en
        else "本地执行、生成证明与验证证明的操作路径。",
    )
    s.text(62, 142, "RUN PATH" if en else "RUN 路径", "section")
    s.text(62, 405, "PROVE PATH" if en else "PROVE 路径", "section")
    s.text(62, 725, "VERIFY PATH" if en else "VERIFY 路径", "section")
    run = [
        ("Parse command" if en else "解析命令", ["script", "arguments", "gas limit"] if en else ["脚本", "参数", "gas 上限"], "blue"),
        ("Build ProofInput" if en else "构造 ProofInput", ["canonical stack values", "bounded script size"] if en else ["规范栈值", "限制脚本大小"], "amber"),
        ("Execute VM" if en else "执行 VM", ["neo-vm-guest", "neo-vm-rs"] if en else ["neo-vm-guest", "neo-vm-rs"], "green"),
        ("Print ProofOutput" if en else "输出 ProofOutput", ["state", "result", "gas", "error"] if en else ["状态", "结果", "gas", "错误"], "teal"),
    ]
    prove = [
        ("Select mode" if en else "选择模式", ["Execute", "Mock", "SP1 / PLONK / Groth16"] if en else ["Execute", "Mock", "SP1 / PLONK / Groth16"], "blue"),
        ("Execute host copy" if en else "执行主机副本", ["derive output", "build public inputs"] if en else ["生成输出", "构造公开输入"], "green"),
        ("Generate proof" if en else "生成证明", ["mock commitment", "or SP1 guest proof"] if en else ["mock 承诺", "或 SP1 guest 证明"], "red"),
        ("Assemble NeoProof" if en else "组装 NeoProof", ["output", "proof bytes", "mode + vkey"] if en else ["输出", "证明字节", "模式 + vkey"], "purple"),
    ]
    verify = [
        ("Load NeoProof" if en else "加载 NeoProof", ["format version", "proof mode"] if en else ["格式版本", "证明模式"], "blue"),
        ("Check bindings" if en else "校验绑定", ["output hash", "gas", "success flag"] if en else ["输出哈希", "gas", "成功标志"], "amber"),
        ("Check proof" if en else "校验证明", ["mock commitment", "or SP1 verifier"] if en else ["mock 承诺", "或 SP1 验证器"], "red"),
        ("Accept / Reject" if en else "接受 / 拒绝", ["fail closed on mismatch"] if en else ["任何不一致都拒绝"], "green"),
    ]

    def lane(items: list[tuple[str, list[str], str]], y: int) -> None:
        x = 62
        for idx, (title, bullet, tone) in enumerate(items):
            s.box(x, y, 270, 170, f"{idx + 1}. {title}", "", bullet, tone)
            if idx < len(items) - 1:
                s.arrow(x + 270, y + 85, x + 330, y + 85, None, "slate")
            x += 360

    lane(run, 165)
    lane(prove, 430)
    lane(verify, 750)
    s.note(
        782,
        630,
        610,
        80,
        "Fallback rule" if en else "降级规则",
        [
            "Production proof modes never downgrade to mock unless fallback is explicitly allowed."
            if en
            else "生产证明模式不会静默降级为 mock，除非显式允许 fallback。"
        ],
        "amber",
    )
    s.poly_arrow([(1142, 600), (1142, 630)], None, "amber", True)
    return s.finish()


def proof_objects(lang: str) -> str:
    en = lang == "en"
    s = Svg(
        1500,
        950,
        "Neo zkVM Proof Objects" if en else "Neo zkVM 证明对象",
        "The data structures that define the proof ABI and verification contract."
        if en
        else "定义证明 ABI 与验证契约的数据结构。",
    )
    s.box(
        70,
        150,
        310,
        225,
        "ProofInput",
        "execution request" if en else "执行请求",
        ["script: Vec<u8>", "arguments: Vec<StackItem>", "gas_limit: u64"],
        "blue",
    )
    s.box(
        445,
        150,
        310,
        225,
        "ProofOutput",
        "execution result" if en else "执行结果",
        ["state: u8", "result: Option<StackItem>", "gas_consumed: u64", "error: Option<String>"],
        "green",
    )
    s.box(
        820,
        150,
        310,
        225,
        "PublicInputs",
        "public verification statement" if en else "公开验证语句",
        ["script_hash: [u8; 32]", "input_hash: [u8; 32]", "output_hash: [u8; 32]", "gas_consumed + success"],
        "purple",
    )
    s.box(
        1175,
        150,
        270,
        225,
        "NeoProof",
        "portable envelope" if en else "可携带封装",
        ["output", "proof_bytes", "public_inputs", "vkey_hash", "proof_mode", "format_version"],
        "red",
    )
    s.arrow(380, 262, 445, 262, "execute", "green")
    s.arrow(755, 262, 820, 262, "hash", "purple")
    s.arrow(1130, 262, 1175, 262, "pack", "red")

    s.box(
        215,
        500,
        430,
        215,
        "StackItem",
        "re-exported canonical value type" if en else "重新导出的规范值类型",
        ["StackItem = neo_vm_rs::StackValue", "same semantics as shared VM core", "no duplicate stack model in neo-zkvm"]
        if en
        else ["StackItem = neo_vm_rs::StackValue", "语义与共享 VM 核心一致", "neo-zkvm 不复制栈模型"],
        "teal",
        "shared type" if en else "共享类型",
    )
    s.box(
        855,
        500,
        430,
        215,
        "MockProof",
        "test-only proof envelope" if en else "仅测试用证明封装",
        ["public_inputs", "commitment", "timestamp"]
        if en
        else ["public_inputs", "commitment", "timestamp"],
        "amber",
        "mock mode" if en else "mock 模式",
    )
    s.poly_arrow([(225, 375), (225, 500)], "arguments", "teal", True)
    s.poly_arrow([(600, 375), (600, 500), (430, 500)], "result", "teal", True)
    s.poly_arrow([(975, 375), (975, 500)], "commitment input", "amber", True)
    s.note(
        70,
        785,
        1375,
        90,
        "ABI guardrails" if en else "ABI 约束",
        [
            "Proof format version is currently 1; oversized inputs are rejected before execution."
            if en
            else "当前证明格式版本为 1；过大的输入在执行前被拒绝。",
            "Serialization uses the canonical bincode configuration shared by guest, prover, and verifier."
            if en
            else "序列化使用 guest、prover、verifier 共享的规范 bincode 配置。",
        ],
        "blue",
    )
    return s.finish()


def verification(lang: str) -> str:
    en = lang == "en"
    s = Svg(
        1520,
        1000,
        "Neo zkVM Verification Logic" if en else "Neo zkVM 验证逻辑",
        "Fail-closed checks for proof format, output binding, mode policy, and cryptographic proof validity."
        if en
        else "对证明格式、输出绑定、模式策略与密码学证明有效性进行失败即拒绝的校验。",
    )
    s.box(70, 145, 220, 105, "NeoProof", "input artifact" if en else "输入产物", ["mode", "version", "output", "public inputs"], "blue")
    s.decision(420, 198, 240, 120, "version == 1?" if en else "version == 1?", "amber")
    s.decision(740, 198, 260, 120, "output matches PI?" if en else "输出匹配 PI?", "purple")
    s.decision(1070, 198, 220, 120, "state == HALT?" if en else "state == HALT?", "green")
    s.box(1295, 145, 165, 105, "Reject", "any failure" if en else "任一失败", ["false"], "red")
    s.arrow(290, 198, 300, 198, None, "blue")
    s.arrow(540, 198, 610, 198, "yes", "green")
    s.arrow(870, 198, 960, 198, "yes", "green")
    s.arrow(1180, 198, 1295, 198, "no", "red")
    s.poly_arrow([(420, 258), (420, 300), (1375, 300), (1375, 250)], "no", "red")
    s.poly_arrow([(740, 258), (740, 300), (1375, 300), (1375, 250)], "no", "red")
    s.poly_arrow([(1070, 258), (1070, 300), (1375, 300), (1375, 250)], "no", "red")

    s.box(
        115,
        430,
        330,
        210,
        "Execute mode" if en else "Execute 模式",
        "deterministic execution only" if en else "仅确定性执行",
        ["no proof bytes required", "accepted only after binding checks"] if en else ["不要求证明字节", "绑定检查通过后才接受"],
        "teal",
    )
    s.box(
        585,
        430,
        330,
        210,
        "Mock mode" if en else "Mock 模式",
        "development and tests" if en else "开发与测试",
        ["decode MockProof", "public inputs must equal", "commitment must recompute"] if en else ["解码 MockProof", "公开输入必须相等", "承诺必须可重算"],
        "amber",
    )
    s.box(
        1055,
        430,
        330,
        210,
        "SP1 / PLONK / Groth16" if en else "SP1 / PLONK / Groth16",
        "production cryptographic modes" if en else "生产密码学模式",
        ["proof type matches mode", "vkey hash matches", "SP1 verifier succeeds", "public values equal PI"]
        if en
        else ["证明类型匹配模式", "vkey hash 匹配", "SP1 验证器成功", "公开值等于 PI"],
        "red",
    )
    s.poly_arrow([(1070, 258), (1070, 345), (280, 345), (280, 430)], "mode branch", "slate")
    s.poly_arrow([(1070, 345), (750, 345), (750, 430)], None, "slate")
    s.poly_arrow([(1070, 345), (1220, 345), (1220, 430)], None, "slate")
    s.box(615, 770, 290, 100, "Valid", "all selected checks passed" if en else "所选检查全部通过", ["true"], "green")
    s.arrow(280, 640, 615, 820, "pass", "green")
    s.arrow(750, 640, 750, 770, "pass", "green")
    s.arrow(1220, 640, 905, 820, "pass", "green")
    s.poly_arrow([(445, 535), (1295, 535)], "fail", "red", True)
    s.poly_arrow([(915, 535), (1295, 535)], "fail", "red", True)
    s.poly_arrow([(1385, 535), (1455, 535), (1455, 250)], "fail", "red", True)
    return s.finish()


def math(lang: str) -> str:
    en = lang == "en"
    s = Svg(
        1540,
        1030,
        "Neo zkVM Mathematical Design" if en else "Neo zkVM 数学设计",
        "Public-input binding and proof statements used by host, guest, and verifier."
        if en
        else "主机、guest 与验证器共同使用的公开输入绑定和证明语句。",
    )
    s.formula_box(
        70,
        145,
        640,
        215,
        "Definitions" if en else "定义",
        [
            "Encode(x) = canonical bincode encoding",
            "H(x) = SHA-256(x)",
            "success = (state == HALT) and (error == None)",
            "LE64(gas) = 8-byte little-endian gas",
        ]
        if en
        else [
            "Encode(x) = 规范 bincode 编码",
            "H(x) = SHA-256(x)",
            "success = (state == HALT) and (error == None)",
            "LE64(gas) = 8 字节小端 gas",
        ],
        "blue",
    )
    s.formula_box(
        830,
        145,
        640,
        215,
        "Public Inputs" if en else "公开输入",
        [
            "h_s = H(script)",
            "h_i = H(Encode(ProofInput))",
            "h_o = H(Encode(ProofOutput))",
            "PI = (h_s, h_i, h_o, gas_consumed, success)",
        ],
        "purple",
    )
    s.formula_box(
        70,
        430,
        640,
        250,
        "Execution Relation" if en else "执行关系",
        [
            "Exec_neo-vm-rs(ProofInput) -> ProofOutput",
            "gas_consumed <= gas_limit",
            "script size <= PROOF_MAX_SCRIPT_SIZE",
            "StackItem semantics = neo_vm_rs::StackValue",
        ]
        if en
        else [
            "Exec_neo-vm-rs(ProofInput) -> ProofOutput",
            "gas_consumed <= gas_limit",
            "script size <= PROOF_MAX_SCRIPT_SIZE",
            "StackItem 语义 = neo_vm_rs::StackValue",
        ],
        "green",
    )
    s.formula_box(
        830,
        430,
        640,
        250,
        "Commitment" if en else "承诺",
        [
            "C = H(h_s || h_i || h_o ||",
            "      LE64(gas_consumed) || byte(success))",
            "MockProof.commitment must equal C",
            "SP1 public values must equal PI",
        ]
        if en
        else [
            "C = H(h_s || h_i || h_o ||",
            "      LE64(gas_consumed) || byte(success))",
            "MockProof.commitment 必须等于 C",
            "SP1 公开值必须等于 PI",
        ],
        "amber",
    )
    s.formula_box(
        70,
        760,
        1400,
        160,
        "Verifier Acceptance Conditions" if en else "验证器接受条件",
        [
            "version == PROOF_FORMAT_VERSION",
            "H(Encode(output)) == PI.output_hash and output.gas_consumed == PI.gas_consumed",
            "mode == Mock: commitment == C; mode in {SP1, PLONK, Groth16}: SP1 verifier accepts proof and vkey hash",
        ]
        if en
        else [
            "version == PROOF_FORMAT_VERSION",
            "H(Encode(output)) == PI.output_hash and output.gas_consumed == PI.gas_consumed",
            "mode == Mock: commitment == C; mode in {SP1, PLONK, Groth16}: SP1 verifier accepts proof and vkey hash",
        ],
        "red",
    )
    s.arrow(710, 252, 830, 252, "hashes", "purple")
    s.arrow(710, 555, 830, 555, "binds", "amber")
    s.poly_arrow([(390, 680), (390, 760)], "output", "green")
    s.poly_arrow([(1150, 680), (1150, 760)], "proof statement", "red")
    return s.finish()


def write_readmes() -> None:
    en = """# Neo zkVM Figures

These diagrams provide a visual companion to the Neo zkVM implementation. They are generated from `docs/figures/generate_figures.py` so the English and Chinese versions stay aligned.

## Figure Set

| Figure | Purpose |
| --- | --- |
| [Architecture](neo-zkvm-architecture.svg) | Crate boundaries, SP1 integration, verifier policy, and the shared `neo-vm-rs` execution core. |
| [Dataflow](neo-zkvm-dataflow.svg) | How a script becomes `ProofInput`, `ProofOutput`, `PublicInputs`, proof bytes, and verifier evidence. |
| [Workflow](neo-zkvm-workflow.svg) | CLI and library paths for `run`, `prove`, and verifier operations. |
| [Proof Objects](neo-zkvm-proof-objects.svg) | The proof ABI structures and how they bind to `neo-vm-rs` types. |
| [Verification Logic](neo-zkvm-verification.svg) | Fail-closed verifier checks for Execute, Mock, SP1, PLONK, and Groth16 modes. |
| [Mathematical Design](neo-zkvm-math.svg) | Hashes, commitments, public inputs, and acceptance conditions. |

## Preview

![Neo zkVM Architecture](neo-zkvm-architecture.svg)

## Other Languages

- [中文图表](../zh/figures/README.md)
"""

    zh = """# Neo zkVM 图表

这些图表是 Neo zkVM 实现的可视化说明。它们由 `docs/figures/generate_figures.py` 生成，因此英文版和中文版保持结构一致。

## 图表集合

| 图表 | 用途 |
| --- | --- |
| [架构](neo-zkvm-architecture.zh.svg) | 展示 crate 边界、SP1 集成、验证器策略，以及共享的 `neo-vm-rs` 执行核心。 |
| [数据流](neo-zkvm-dataflow.zh.svg) | 展示脚本如何变成 `ProofInput`、`ProofOutput`、`PublicInputs`、证明字节和验证证据。 |
| [工作流](neo-zkvm-workflow.zh.svg) | 展示 `run`、`prove` 和验证路径在 CLI 与库中的操作流程。 |
| [证明对象](neo-zkvm-proof-objects.zh.svg) | 展示证明 ABI 结构，以及它们如何绑定到 `neo-vm-rs` 类型。 |
| [验证逻辑](neo-zkvm-verification.zh.svg) | 展示 Execute、Mock、SP1、PLONK、Groth16 模式下失败即拒绝的验证检查。 |
| [数学设计](neo-zkvm-math.zh.svg) | 展示哈希、承诺、公开输入和验证器接受条件。 |

## 预览

![Neo zkVM 架构](neo-zkvm-architecture.zh.svg)

## 其他语言

- [English figures](../../figures/README.md)
"""

    zh_index = """# Neo zkVM 中文文档

- [Neo zkVM 图表](figures/README.md)
"""

    EN_DIR.mkdir(parents=True, exist_ok=True)
    ZH_DIR.mkdir(parents=True, exist_ok=True)
    (EN_DIR / "README.md").write_text(en, encoding="utf-8", newline="\n")
    (ZH_DIR / "README.md").write_text(zh, encoding="utf-8", newline="\n")
    (DOCS / "zh" / "README.md").write_text(zh_index, encoding="utf-8", newline="\n")


def main() -> None:
    EN_DIR.mkdir(parents=True, exist_ok=True)
    ZH_DIR.mkdir(parents=True, exist_ok=True)
    figures = [
        ("neo-zkvm-architecture", architecture),
        ("neo-zkvm-dataflow", dataflow),
        ("neo-zkvm-workflow", workflow),
        ("neo-zkvm-proof-objects", proof_objects),
        ("neo-zkvm-verification", verification),
        ("neo-zkvm-math", math),
    ]
    for name, builder in figures:
        (EN_DIR / f"{name}.svg").write_text(builder("en"), encoding="utf-8", newline="\n")
        (ZH_DIR / f"{name}.zh.svg").write_text(builder("zh"), encoding="utf-8", newline="\n")
    write_readmes()


if __name__ == "__main__":
    main()
