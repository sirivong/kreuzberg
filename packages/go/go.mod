module github.com/xberg-io/xberg/packages/go

go 1.26

// NOTE (issue #1230): `go get github.com/xberg-io/xberg@latest` (bare repository root,
// without the /packages/go suffix) resolves to a bogus v4.9.9+incompatible pseudo-version
// and fails with "module source tree too large". That version was never published under
// this module path — it is a Go module-proxy cache artifact left over from the
// kreuzberg-dev era, before the repository was renamed/restructured. The proxy cache
// cannot be purged, and the repository root is intentionally not a Go module (this
// package lives at /packages/go on purpose). There is nothing to retract here: no
// versions have ever been published under github.com/xberg-io/xberg/packages/go that
// need to be pulled back. The supported install path is:
//
//	go get github.com/xberg-io/xberg/packages/go@latest
