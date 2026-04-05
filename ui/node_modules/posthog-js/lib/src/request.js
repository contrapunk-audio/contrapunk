"use strict";
var __assign = (this && this.__assign) || function () {
    __assign = Object.assign || function(t) {
        for (var s, i = 1, n = arguments.length; i < n; i++) {
            s = arguments[i];
            for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p))
                t[p] = s[p];
        }
        return t;
    };
    return __assign.apply(this, arguments);
};
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype);
    return g.next = verb(0), g["throw"] = verb(1), g["return"] = verb(2), typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
var __read = (this && this.__read) || function (o, n) {
    var m = typeof Symbol === "function" && o[Symbol.iterator];
    if (!m) return o;
    var i = m.call(o), r, ar = [], e;
    try {
        while ((n === void 0 || n-- > 0) && !(r = i.next()).done) ar.push(r.value);
    }
    catch (error) { e = { error: error }; }
    finally {
        try {
            if (r && !r.done && (m = i["return"])) m.call(i);
        }
        finally { if (e) throw e.error; }
    }
    return ar;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.request = exports.jsonStringify = exports.extendURLParams = exports.SUPPORTS_REQUEST = void 0;
var utils_1 = require("./utils");
var config_1 = __importDefault(require("./config"));
var types_1 = require("./types");
var request_utils_1 = require("./utils/request-utils");
var logger_1 = require("./utils/logger");
var globals_1 = require("./utils/globals");
var fflate_1 = require("fflate");
var encode_utils_1 = require("./utils/encode-utils");
var core_1 = require("@posthog/core");
// eslint-disable-next-line compat/compat
exports.SUPPORTS_REQUEST = !!globals_1.XMLHttpRequest || !!globals_1.fetch;
var CONTENT_TYPE_PLAIN = 'text/plain';
var CONTENT_TYPE_JSON = 'application/json';
var CONTENT_TYPE_FORM = 'application/x-www-form-urlencoded';
var SIXTY_FOUR_KILOBYTES = 64 * 1024;
/*
 fetch will fail if we request keepalive with a body greater than 64kb
 sets the threshold lower than that so that
 any overhead doesn't push over the threshold after checking here
*/
var KEEP_ALIVE_THRESHOLD = SIXTY_FOUR_KILOBYTES * 0.8;
/**
 * Extends a URL with additional query parameters
 * @param url - The URL to extend
 * @param params - The parameters to add
 * @param replace - When true (default), new params overwrite existing ones with same key. When false, existing params are preserved.
 * @returns The URL with extended parameters
 */
var extendURLParams = function (url, params, replace) {
    var _a;
    if (replace === void 0) { replace = true; }
    var _b = __read(url.split('?'), 2), baseUrl = _b[0], search = _b[1];
    var newParams = __assign({}, params);
    var updatedSearch = (_a = search === null || search === void 0 ? void 0 : search.split('&').map(function (pair) {
        var _a;
        var _b = __read(pair.split('='), 2), key = _b[0], origValue = _b[1];
        var value = replace ? ((_a = newParams[key]) !== null && _a !== void 0 ? _a : origValue) : origValue;
        delete newParams[key];
        return "".concat(key, "=").concat(value);
    })) !== null && _a !== void 0 ? _a : [];
    var remaining = (0, request_utils_1.formDataToQuery)(newParams);
    if (remaining) {
        updatedSearch.push(remaining);
    }
    return "".concat(baseUrl, "?").concat(updatedSearch.join('&'));
};
exports.extendURLParams = extendURLParams;
var jsonStringify = function (data, space) {
    // With plain JSON.stringify, we get an exception when a property is a BigInt. This has caused problems for some users,
    // see https://github.com/PostHog/posthog-js/issues/1440
    // To work around this, we convert BigInts to strings before stringifying the data. This is not ideal, as we lose
    // information that this was originally a number, but given ClickHouse doesn't support BigInts, the customer
    // would not be able to operate on these numerically anyway.
    return JSON.stringify(data, function (_, value) { return (typeof value === 'bigint' ? value.toString() : value); }, space);
};
exports.jsonStringify = jsonStringify;
var encodeToDataString = function (data) {
    return 'data=' + encodeURIComponent(typeof data === 'string' ? data : (0, exports.jsonStringify)(data));
};
var encodePostData = function (options) {
    // Use pre-encoded body if available (set by async compression in the request entrypoint)
    if (options._encodedBody) {
        return options._encodedBody;
    }
    var data = options.data, compression = options.compression;
    if (!data) {
        return;
    }
    if (compression === types_1.Compression.GZipJS) {
        var gzipData = (0, fflate_1.gzipSync)((0, fflate_1.strToU8)((0, exports.jsonStringify)(data)), { mtime: 0 });
        return {
            contentType: CONTENT_TYPE_PLAIN,
            body: gzipData.buffer.slice(gzipData.byteOffset, gzipData.byteOffset + gzipData.byteLength),
            estimatedSize: gzipData.byteLength,
        };
    }
    if (compression === types_1.Compression.Base64) {
        var b64data = (0, encode_utils_1._base64Encode)((0, exports.jsonStringify)(data));
        var encodedBody = encodeToDataString(b64data);
        return {
            contentType: CONTENT_TYPE_FORM,
            body: encodedBody,
            estimatedSize: new Blob([encodedBody]).size,
        };
    }
    var jsonBody = (0, exports.jsonStringify)(data);
    return {
        contentType: CONTENT_TYPE_JSON,
        body: jsonBody,
        estimatedSize: new Blob([jsonBody]).size,
    };
};
/**
 * Pre-encode the request body using async native CompressionStream.
 * This avoids blocking the main thread with fflate's synchronous gzip,
 * which can take 300ms+ on constrained devices.
 *
 * Callers must check preconditions (data exists, gzip compression, CompressionStream available)
 * before calling this function. Uses `gzipCompress` from @posthog/core.
 */
var preEncodeAsync = function (options) { return __awaiter(void 0, void 0, void 0, function () {
    var jsonData, compressed, body;
    return __generator(this, function (_a) {
        switch (_a.label) {
            case 0:
                jsonData = (0, exports.jsonStringify)(options.data);
                return [4 /*yield*/, (0, core_1.gzipCompress)(jsonData, config_1.default.DEBUG)];
            case 1:
                compressed = _a.sent();
                if (!compressed) {
                    return [2 /*return*/, options];
                }
                return [4 /*yield*/, compressed.arrayBuffer()];
            case 2:
                body = _a.sent();
                return [2 /*return*/, __assign(__assign({}, options), { _encodedBody: {
                            contentType: CONTENT_TYPE_PLAIN,
                            body: body,
                            estimatedSize: body.byteLength,
                        } })];
        }
    });
}); };
var xhr = function (options) {
    var _a;
    var req = new globals_1.XMLHttpRequest();
    req.open(options.method || 'GET', options.url, true);
    var _b = (_a = encodePostData(options)) !== null && _a !== void 0 ? _a : {}, contentType = _b.contentType, body = _b.body;
    (0, utils_1.each)(options.headers, function (headerValue, headerName) {
        req.setRequestHeader(headerName, headerValue);
    });
    if (contentType) {
        req.setRequestHeader('Content-Type', contentType);
    }
    if (options.timeout) {
        req.timeout = options.timeout;
    }
    if (!options.disableXHRCredentials) {
        // send the ph_optout cookie
        // withCredentials cannot be modified until after calling .open on Android and Mobile Safari
        req.withCredentials = true;
    }
    req.onreadystatechange = function () {
        var _a;
        // XMLHttpRequest.DONE == 4, except in safari 4
        if (req.readyState === 4) {
            var response = {
                statusCode: req.status,
                text: req.responseText,
            };
            if (req.status === 200) {
                try {
                    response.json = JSON.parse(req.responseText);
                }
                catch (_b) {
                    // logger.error(e)
                }
            }
            (_a = options.callback) === null || _a === void 0 ? void 0 : _a.call(options, response);
        }
    };
    req.send(body);
};
var _fetch = function (options) {
    var _a;
    var _b = (_a = encodePostData(options)) !== null && _a !== void 0 ? _a : {}, contentType = _b.contentType, body = _b.body, estimatedSize = _b.estimatedSize;
    // eslint-disable-next-line compat/compat
    var headers = new Headers();
    (0, utils_1.each)(options.headers, function (headerValue, headerName) {
        headers.append(headerName, headerValue);
    });
    if (contentType) {
        headers.append('Content-Type', contentType);
    }
    var url = options.url;
    var aborter = null;
    if (globals_1.AbortController) {
        var controller_1 = new globals_1.AbortController();
        aborter = {
            signal: controller_1.signal,
            timeout: setTimeout(function () { return controller_1.abort(); }, options.timeout),
        };
    }
    globals_1.fetch(url, __assign({ method: (options === null || options === void 0 ? void 0 : options.method) || 'GET', headers: headers, 
        // if body is greater than 64kb, then fetch with keepalive will error
        // see 8:10:5 at https://fetch.spec.whatwg.org/#http-network-or-cache-fetch,
        // but we do want to set keepalive sometimes as it can  help with success
        // when e.g. a page is being closed
        // so let's get the best of both worlds and only set keepalive for POST requests
        // where the body is less than 64kb
        // NB this is fetch keepalive and not http keepalive
        keepalive: options.method === 'POST' && (estimatedSize || 0) < KEEP_ALIVE_THRESHOLD, body: body, signal: aborter === null || aborter === void 0 ? void 0 : aborter.signal }, options.fetchOptions))
        .then(function (response) {
        return response.text().then(function (responseText) {
            var _a;
            var res = {
                statusCode: response.status,
                text: responseText,
            };
            if (response.status === 200) {
                try {
                    res.json = JSON.parse(responseText);
                }
                catch (e) {
                    logger_1.logger.error(e);
                }
            }
            (_a = options.callback) === null || _a === void 0 ? void 0 : _a.call(options, res);
        });
    })
        .catch(function (error) {
        var _a;
        logger_1.logger.error(error);
        (_a = options.callback) === null || _a === void 0 ? void 0 : _a.call(options, { statusCode: 0, error: error });
    })
        .finally(function () { return (aborter ? clearTimeout(aborter.timeout) : null); });
    return;
};
var _sendBeacon = function (options) {
    // beacon documentation https://w3c.github.io/beacon/
    // beacons format the message and use the type property
    var _a;
    var url = (0, exports.extendURLParams)(options.url, {
        beacon: '1',
    });
    try {
        var _b = (_a = encodePostData(options)) !== null && _a !== void 0 ? _a : {}, contentType = _b.contentType, body = _b.body;
        if (!body) {
            return;
        }
        // sendBeacon requires a Blob to set the Content-Type header correctly.
        // Without wrapping, ArrayBuffer bodies are sent with no Content-Type,
        // which can cause issues with proxies/WAFs that require it.
        var sendBeaconBody = body instanceof Blob ? body : new Blob([body], { type: contentType });
        globals_1.navigator.sendBeacon(url, sendBeaconBody);
    }
    catch (_c) {
        // send beacon is a best-effort, fire-and-forget mechanism on page unload,
        // we don't want to throw errors here
    }
};
var AVAILABLE_TRANSPORTS = [];
// We add the transports in order of preference
if (globals_1.fetch) {
    AVAILABLE_TRANSPORTS.push({
        transport: 'fetch',
        method: _fetch,
    });
}
if (globals_1.XMLHttpRequest) {
    AVAILABLE_TRANSPORTS.push({
        transport: 'XHR',
        method: xhr,
    });
}
if (globals_1.navigator === null || globals_1.navigator === void 0 ? void 0 : globals_1.navigator.sendBeacon) {
    AVAILABLE_TRANSPORTS.push({
        transport: 'sendBeacon',
        method: _sendBeacon,
    });
}
// This is the entrypoint. It takes care of sanitizing the options and then calls the appropriate request method.
var request = function (_options) {
    var _a, _b, _c;
    // Clone the options so we don't modify the original object
    var options = __assign({}, _options);
    options.timeout = options.timeout || 60000;
    options.url = (0, exports.extendURLParams)(options.url, {
        _: new Date().getTime().toString(),
        ver: config_1.default.JS_SDK_VERSION,
        compression: options.compression,
    });
    var transport = (_a = options.transport) !== null && _a !== void 0 ? _a : 'fetch';
    var availableTransports = AVAILABLE_TRANSPORTS.filter(function (t) { return !options.disableTransport || !t.transport || !options.disableTransport.includes(t.transport); });
    var transportMethod = (_c = (_b = (0, utils_1.find)(availableTransports, function (t) { return t.transport === transport; })) === null || _b === void 0 ? void 0 : _b.method) !== null && _c !== void 0 ? _c : availableTransports[0].method;
    if (!transportMethod) {
        throw new Error('No available transport method');
    }
    // For non-sendBeacon transports, use async native CompressionStream when available
    // to avoid blocking the main thread with fflate's synchronous gzip (which can take 300ms+).
    // sendBeacon must remain synchronous as it's used during page unload.
    if (transport !== 'sendBeacon' &&
        options.data &&
        options.compression === types_1.Compression.GZipJS &&
        !!globals_1.CompressionStream) {
        preEncodeAsync(options)
            .then(function (encodedOptions) {
            transportMethod(encodedOptions);
        })
            .catch(function () {
            // If async compression fails, fall back to the synchronous fflate path
            transportMethod(options);
        });
    }
    else {
        transportMethod(options);
    }
};
exports.request = request;
//# sourceMappingURL=request.js.map