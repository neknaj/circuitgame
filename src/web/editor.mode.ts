const oop = ace.require("ace/lib/oop");
const TextMode = ace.require("ace/mode/text").Mode;
const TextHighlightRules = ace.require("ace/mode/text_highlight_rules").TextHighlightRules;

// カスタムHighlightRulesの定義
class CustomHighlightRules extends TextHighlightRules {
    constructor() {
        super();

        this.$rules = {
            start: [
                {
                    token: "keyword.control",
                    regex: "\\b(?:using|Using|USING|use|Use|USE|module|Module|MODULE|def|Def|DEF|func|Func|FUNC|fn|Fn|FN|test|Test|TEST)\\b",
                },

                {
                    token: "boolean",
                    regex: "\\b(?:t|T|h|H|f|F|l|L)\\b",
                },

                {
                    token: "paren",
                    regex: "(?:\\(|\\)|\\[|\\]|\\{|\\})",
                },

                {
                    token: "arrow",
                    regex: "(?:->|=>|~>|>|<-|<=|<~|<)",
                },

                {
                    token: "constant.numeric",
                    regex: "\\b[0-9]+\\b",
                },

                {
                    token: "identifier",
                    regex: "\\b[0-9a-zA-Z_]+\\b",
                },

                {
                    token: "comment.line",
                    regex: "(?:#|//).*",
                },

                {
                    token: "keyword.separator",
                    regex: "[:=]|::=|,",
                },
            ],
        };

        // コメントやスペースの無視を設定
        this.normalizeRules();
    }
}

// カスタムモードの定義
export class CustomMode extends TextMode {
    constructor() {
        super();
        this.HighlightRules = CustomHighlightRules;
    }
}

export function darkTheme (require, exports, module, dom) {
    exports.isDark = true;  // ダークテーマの場合

    exports.cssClass = "ace-ncg-dark";  // CSSクラスの設定

    exports.cssText = `
        .ace-ncg-dark {
            background-color:rgb(0, 0, 0);
            color: #f8f8f2;
        }

        .ace-ncg-dark .ace_keyword.ace_control {
            color:rgb(255, 147, 187);
        }
        .ace-ncg-dark .ace_keyword.ace_separator {
            color:rgb(104, 100, 101);
        }

        .ace-ncg-dark .ace_identifier {
            color: #66d9ef;
        }

        .ace-ncg-dark .ace_boolean {
            color:rgb(126, 175, 255);
        }

        .ace-ncg-dark .ace_constant.ace_numeric {
            color:rgb(151, 255, 137);
        }

        .ace-ncg-dark .ace_comment {
            color:rgb(97, 117, 94);
        }

        .ace-ncg-dark .ace_paren {
            color:rgb(146, 154, 130);
        }

        .ace-ncg-dark .ace_arrow {
            color:rgb(130, 131, 154);
        }

        /* 選択範囲 */
        .ace_selection {
            background-color:rgb(14, 58, 68);
        }
    `;
};