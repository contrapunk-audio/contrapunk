"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.transformEnabledFeatureFlagsToEventProperties = void 0;
var core_1 = require("@posthog/core");
var transformEnabledFeatureFlagsToEventProperties = function (value) {
    if (!(0, core_1.isObject)(value)) {
        return {};
    }
    var eventProperties = {};
    var keys = Object.keys(value);
    for (var i = 0; i < keys.length; i++) {
        eventProperties["$feature/".concat(keys[i])] = value[keys[i]];
    }
    return eventProperties;
};
exports.transformEnabledFeatureFlagsToEventProperties = transformEnabledFeatureFlagsToEventProperties;
//# sourceMappingURL=persistence-key-transforms.js.map