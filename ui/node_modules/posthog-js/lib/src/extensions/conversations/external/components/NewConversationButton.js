"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.NewConversationButton = void 0;
var jsx_runtime_1 = require("preact/jsx-runtime");
/**
 * Primary CTA used anywhere the user can start a fresh conversation —
 * the bottom of the ticket list and the resolved-state banner in the message view.
 */
var NewConversationButton = function (_a) {
    var styles = _a.styles, onClick = _a.onClick;
    return ((0, jsx_runtime_1.jsxs)("button", { type: "button", style: styles.newConversationButton, onClick: onClick, onMouseEnter: function (e) {
            e.currentTarget.style.opacity = '0.9';
        }, onMouseLeave: function (e) {
            e.currentTarget.style.opacity = '1';
        }, children: [(0, jsx_runtime_1.jsxs)("svg", { width: "16", height: "16", viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: "2", style: { marginRight: '8px' }, children: [(0, jsx_runtime_1.jsx)("line", { x1: "12", y1: "5", x2: "12", y2: "19" }), (0, jsx_runtime_1.jsx)("line", { x1: "5", y1: "12", x2: "19", y2: "12" })] }), "New conversation"] }));
};
exports.NewConversationButton = NewConversationButton;
//# sourceMappingURL=NewConversationButton.js.map