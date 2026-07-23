module.exports = {
  root: true,
  env: {
    node: true,
    es2022: true,
  },
  parser: "@typescript-eslint/parser",
  parserOptions: {
    sourceType: "module",
    ecmaVersion: 2022,
  },
  plugins: ["@typescript-eslint"],
  extends: ["eslint:recommended", "plugin:@typescript-eslint/recommended"],
  ignorePatterns: ["dist/**", "node_modules/**", ".eslintrc.cjs", "tsup.config.ts"],
  rules: {
    "@typescript-eslint/no-explicit-any": "off",
  },
};
