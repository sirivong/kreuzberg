# Kreuzberg PHP Tests

This directory contains the test suite for the Kreuzberg PHP bindings.

## Directory Structure

```
tests/
├── bootstrap.php           # PHPUnit bootstrap file
├── Unit/                   # Unit tests
│   ├── ConfigTest.php     # Configuration class tests
│   └── ExtractionTest.php # Extraction API tests
└── Integration/           # Integration tests
    └── ExtensionTest.php  # Extension loading and FFI tests
```

## Running Tests

### All Tests

```bash
composer test
# or
vendor/bin/phpunit
```

### Unit Tests Only

```bash
vendor/bin/phpunit --testsuite Unit
```

### Integration Tests Only

```bash
vendor/bin/phpunit --testsuite Integration
```

### Specific Test File

```bash
vendor/bin/phpunit tests/Unit/ConfigTest.php
```

### With Coverage

```bash
vendor/bin/phpunit --coverage-html coverage/
```

## Test Organization

### Unit Tests (`tests/Unit/`)

Unit tests verify individual classes and methods in isolation without requiring the Kreuzberg extension to be loaded.

- **ConfigTest.php** - Tests all configuration classes including:
  - `ExtractionConfig`
  - `OcrConfig`
  - `PdfConfig`
  - `ChunkingConfig`
  - `TesseractConfig`
  - And other config objects

- **ExtractionTest.php** - Tests the core Kreuzberg API:
  - `Kreuzberg` class instantiation
  - Method signatures and availability
  - Version information
  - API structure validation

### Integration Tests (`tests/Integration/`)

Integration tests verify the PHP extension integration with the Rust core. These tests require the Kreuzberg extension to be loaded.

- **ExtensionTest.php** - Tests extension functionality:
  - Extension loading verification
  - Native function availability
  - MIME type detection
  - Configuration serialization
  - Error handling
  - Batch operation signatures

**Note**: Integration tests will be automatically skipped if the Kreuzberg extension is not loaded.

## Extension Requirements

The integration tests require the Kreuzberg PHP extension to be loaded. Tests will gracefully skip if the extension is not available.

Check if the extension is loaded:

```bash
php -m | grep kreuzberg
```

Get extension version:

```bash
php -r "echo phpversion('kreuzberg');"
```

## PHPUnit 11 Features Used

- **Attributes** - Using PHP 8+ attributes for test metadata:
  - `#[Test]` - Mark test methods
  - `#[CoversClass]` - Specify covered classes
  - `#[Group]` - Organize tests by group
  - `#[RequiresPhpExtension]` - Skip tests if extension missing

- **Strict Mode** - Tests run with strict assertions and error checking
- **Type Safety** - Full type hints on all test methods
- **Readonly Classes** - Tests verify readonly class behavior

## Writing New Tests

### Unit Test Template

```php
<?php

declare(strict_types=1);

namespace Kreuzberg\Tests\Unit;

use PHPUnit\Framework\Attributes\CoversClass;
use PHPUnit\Framework\Attributes\Test;
use PHPUnit\Framework\TestCase;

#[CoversClass(YourClass::class)]
final class YourClassTest extends TestCase
{
    #[Test]
    public function it_does_something(): void
    {
        // Arrange
        $object = new YourClass();

        // Act
        $result = $object->doSomething();

        // Assert
        $this->assertTrue($result);
    }
}
```

### Integration Test Template

```php
<?php

declare(strict_types=1);

namespace Kreuzberg\Tests\Integration;

use PHPUnit\Framework\Attributes\Group;
use PHPUnit\Framework\Attributes\RequiresPhpExtension;
use PHPUnit\Framework\Attributes\Test;
use PHPUnit\Framework\TestCase;

#[Group('integration')]
#[RequiresPhpExtension('kreuzberg')]
final class YourIntegrationTest extends TestCase
{
    protected function setUp(): void
    {
        if (!extension_loaded('kreuzberg')) {
            $this->markTestSkipped('Kreuzberg extension not loaded');
        }
    }

    #[Test]
    public function it_integrates_with_extension(): void
    {
        // Your test code
    }
}
```

## Best Practices

1. **Use descriptive test names** - Test methods should read like sentences:
   - ✅ `it_creates_config_with_default_values()`
   - ❌ `testConfig()`

2. **One assertion per test** - Keep tests focused and specific

3. **Arrange-Act-Assert** - Structure tests clearly:
   ```php
   // Arrange - Set up test data
   $config = new ExtractionConfig();

   // Act - Execute the operation
   $result = $config->toArray();

   // Assert - Verify the outcome
   $this->assertIsArray($result);
   ```

4. **Test edge cases** - Include tests for:
   - Null values
   - Empty arrays
   - Invalid input
   - Boundary conditions

5. **Use type hints** - All test methods should use strict types

6. **Skip gracefully** - Use `markTestSkipped()` for tests that can't run in current environment

## Continuous Integration

Tests are automatically run on:
- Pull requests
- Commits to main branch
- Release builds

CI runs tests with and without the Kreuzberg extension to ensure graceful degradation.
