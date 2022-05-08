const { merge } = require('webpack-merge');
const common = require('./webpack.common.js');

module.exports = merge(common, {
    mode: 'production',
    cache: {
        type: 'filesystem',
        hashAlgorithm: 'sha256',
    },
});
