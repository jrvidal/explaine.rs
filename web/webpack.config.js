const webpack = require("webpack");
const MiniCssExtractPlugin = require("mini-css-extract-plugin");
const OptimizeCSSAssetsPlugin = require("optimize-css-assets-webpack-plugin");

const env = require("../env.json");
const devConfig = require("./webpack.config.dev");

module.exports = {
  mode: "production",
  entry: {
    index: "./web/index.ts",
    style: "./web/style.css",
  },
  output: {
    filename: "[name]-[contenthash].js",
    globalObject: "this",
  },
  resolve: devConfig.resolve,
  module: {
    rules: [
      {
        test: /\.css$/i,
        use: [MiniCssExtractPlugin.loader, "css-loader"],
      },
      ...devConfig.module.rules,
    ],
  },
  plugins: [
    new MiniCssExtractPlugin({
      filename: "[name]-[contenthash].css",
    }),
    new OptimizeCSSAssetsPlugin(),
    new webpack.DefinePlugin({
      "self.__ANALYTICS_URL__": JSON.stringify(
        env.analyticsUrl ||
          (() => {
            throw new Error("No analyticsUrl");
          })()
      ),
      "self.__PRODUCTION__": JSON.stringify(true),
    }),
  ],
};
