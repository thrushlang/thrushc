(function () {
  "use strict";

  const KEYWORDS = [
    "if", "elif", "else", "for", "while", "loop", "return", "break", "continue",
    "breakall", "continueall", "pass", "defer", "unreachable", "fn", "var", "const",
    "struct", "enum", "type", "asmfn", "intrinsic", "embedded", "import", "importC",
    "new", "only", "as", "deref", "ref", "address", "addr", "load", "write", "alloc",
    "fixed", "or", "and", "mut", "static", "global_asm", "asm", "true", "false", "nullptr"
  ];

  const TYPES = [
    "s8", "s16", "s32", "s64", "ssize", "u8", "u16", "u32", "u64", "u128", "usize",
    "f32", "f64", "f128", "f80", "fppc_128", "bool", "char", "ptr", "array", "void", "Fn", "CString"
  ];

  const BUILTINS = [
    "halloc", "sizeOf", "memset", "memmove", "memcpy", "alignOf", "abiSizeOf", "bitSizeOf", "abiAlignOf",
    "arbitraryArg", "arbitraryArgs", "staticAssert", "compileError", "compileWarning", "file", "fileLine",
    "currentFuncName", "isSigned", "isUnsigned", "isInteger", "isFloat", "isBool", "isChar", "isPointer",
    "isArray", "isFixedArray", "isStruct", "isVoid", "isConst", "isNumeric", "isFunction", "typeWidth",
    "fieldCount", "fixedArraySize", "isSameType", "isPtrLike", "isFixedArrayOfSize", "compilerVersion",
    "debugBuild", "stringLength", "targetOS", "targetArch", "targetVendor", "targetAbi", "targetTriple",
    "isLinux", "isWindows", "isDarwin", "isApple", "isAix", "is64Bit", "is32Bit", "isBigEndian",
    "isLittleEndian", "isX86", "isX8664", "isArm", "isAarch64", "isRiscv64", "isPpc", "isPpc64",
    "isMips64", "isSystemz", "isLoongarch64", "isWasm", "isElf", "isMachO", "isCoff", "hasPosixThreads",
    "hasSysvAbi", "pointerWidth"
  ];

  const ATTRIBUTES = [
    "asmAlignStack", "asmSyntax", "asmThrowErrors", "asmSideEffects", "align", "optFuzzing", "noUnwind",
    "noReturn", "packed", "heap", "public", "linkage", "extern", "arbitraryArgs", "hot", "minSize",
    "alwaysInline", "noInline", "inline", "safeStack", "weakStack", "strongStack", "preciseFloatingPoint",
    "convention", "pure", "thunk", "cuda", "constructor", "destructor", "if", "elif", "else", "promote"
  ];

  function escapeHtml(text) {
    return text
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }

  const KEYWORD_SET = new Set(KEYWORDS);
  const TYPE_SET = new Set(TYPES);
  const BUILTIN_SET = new Set(BUILTINS);
  const ATTRIBUTE_SET = new Set(ATTRIBUTES);

  function highlight(code) {
    let i = 0;
    let out = "";

    while (i < code.length) {
      const rest = code.slice(i);

      const lineComment = rest.match(/^\/\/.*(?:\n|$)/);
      if (lineComment) {
        out += '<span class="th-comment">' + escapeHtml(lineComment[0]) + "</span>";
        i += lineComment[0].length;
        continue;
      }

      const blockComment = rest.match(/^\/\*[\s\S]*?\*\//);
      if (blockComment) {
        out += '<span class="th-comment">' + escapeHtml(blockComment[0]) + "</span>";
        i += blockComment[0].length;
        continue;
      }

      const str = rest.match(/^"(?:\\.|[^"\\])*"/);
      if (str) {
        out += '<span class="th-string">' + escapeHtml(str[0]) + "</span>";
        i += str[0].length;
        continue;
      }

      const chr = rest.match(/^'(?:\\.|[^'\\])'/);
      if (chr) {
        out += '<span class="th-char">' + escapeHtml(chr[0]) + "</span>";
        i += chr[0].length;
        continue;
      }

      const attrToken = rest.match(/^@([A-Za-z_][A-Za-z0-9_]*)/);
      if (attrToken) {
        const attrName = attrToken[1];
        const fullAttr = attrToken[0];
        if (ATTRIBUTE_SET.has(attrName)) {
          out += '<span class="th-attr">' + escapeHtml(fullAttr) + "</span>";
        } else {
          out += escapeHtml(fullAttr);
        }
        i += fullAttr.length;
        continue;
      }

      const number = rest.match(/^(0x[0-9a-fA-F][0-9a-fA-F_]*|0b[01][01_]*|[0-9][0-9_]*(?:\.[0-9][0-9_]*)?)/);
      if (number) {
        out += '<span class="th-number">' + number[0] + "</span>";
        i += number[0].length;
        continue;
      }

      const ident = rest.match(/^[A-Za-z_][A-Za-z0-9_]*/);
      if (ident) {
        const name = ident[0];
        if (KEYWORD_SET.has(name)) {
          out += '<span class="th-keyword">' + name + "</span>";
        } else if (TYPE_SET.has(name)) {
          out += '<span class="th-type">' + name + "</span>";
        } else if (BUILTIN_SET.has(name)) {
          out += '<span class="th-builtin">' + name + "</span>";
        } else if (/^[A-Z][A-Z0-9_]{1,}$/.test(name)) {
          out += '<span class="th-constant">' + name + "</span>";
        } else {
          out += escapeHtml(name);
        }
        i += name.length;
        continue;
      }

      const op = rest.match(/^(\+=|-=|\*=|\/=|%=|<<=|>>=|&=|\|=|\^=|&&|\|\||==|!=|<=|>=|\+\+|--|::|->|=>|\.\.|<<|>>|[=<>+\-*/%&|^~!])/);
      if (op) {
        out += '<span class="th-op">' + escapeHtml(op[0]) + "</span>";
        i += op[0].length;
        continue;
      }

      out += escapeHtml(code[i]);
      i += 1;
    }

    return out;
  }

  const blocks = document.querySelectorAll("pre.code-thrust code.language-thrust:not([data-no-highlight])");
  blocks.forEach(function (block) {
    const source = block.textContent || "";
    block.innerHTML = highlight(source);
  });
})();
