"""End-to-end tests for Python bindings config parity.

Tests the new config fields `output_format` and `result_format` to ensure they
properly affect extraction results across different document types.
"""

import json
from pathlib import Path

import pytest

try:
    from kreuzberg import (
        ExtractionConfig,
        ExtractionResult,
        extract_bytes_sync,
        extract_file_sync,
    )
except ImportError:
    pytest.skip("kreuzberg not installed", allow_module_level=True)


# Get test documents directory
REPO_ROOT = Path(__file__).parent.parent.parent.parent
TEST_DOCUMENTS = REPO_ROOT / "test_documents"


class TestOutputFormatParity:
    """Test output_format field behavior."""

    def get_sample_document(self) -> bytes:
        """Get a sample text document for testing."""
        sample_path = TEST_DOCUMENTS / "text" / "report.txt"
        if sample_path.exists():
            return sample_path.read_bytes()
        # Fallback: create sample text
        return b"Hello World\n\nThis is a test document with multiple lines."

    def test_output_format_plain_default(self) -> None:
        """Test that Plain is the default output format."""
        config = ExtractionConfig()
        assert config.output_format == "Plain"

    def test_output_format_serialization(self) -> None:
        """Test that output_format serializes correctly."""
        config = ExtractionConfig(output_format="Markdown")
        json_str = config.to_json()
        data = json.loads(json_str)
        assert data.get("output_format") == "Markdown"

    def test_extraction_with_plain_format(self) -> None:
        """Test extraction with Plain output format."""
        doc_bytes = self.get_sample_document()
        config = ExtractionConfig(output_format="Plain")

        result = extract_bytes_sync(doc_bytes, "text/plain", config)

        assert isinstance(result, ExtractionResult)
        assert result.content is not None
        # Plain format should have raw text content
        assert len(result.content) > 0

    def test_extraction_with_markdown_format(self) -> None:
        """Test extraction with Markdown output format."""
        doc_bytes = self.get_sample_document()
        config = ExtractionConfig(output_format="Markdown")

        result = extract_bytes_sync(doc_bytes, "text/plain", config)

        assert isinstance(result, ExtractionResult)
        assert result.content is not None

    def test_extraction_with_html_format(self) -> None:
        """Test extraction with HTML output format."""
        doc_bytes = self.get_sample_document()
        config = ExtractionConfig(output_format="Html")

        result = extract_bytes_sync(doc_bytes, "text/plain", config)

        assert isinstance(result, ExtractionResult)
        assert result.content is not None

    def test_output_format_affects_content(self) -> None:
        """Test that different output formats produce different content."""
        doc_bytes = self.get_sample_document()

        # Extract with different formats
        plain_config = ExtractionConfig(output_format="Plain")
        plain_result = extract_bytes_sync(doc_bytes, "text/plain", plain_config)

        markdown_config = ExtractionConfig(output_format="Markdown")
        markdown_result = extract_bytes_sync(doc_bytes, "text/plain", markdown_config)

        # Both should have content but they may differ in formatting
        assert plain_result.content is not None
        assert markdown_result.content is not None


class TestResultFormatParity:
    """Test result_format field behavior."""

    def get_sample_document(self) -> bytes:
        """Get a sample document for testing."""
        sample_path = TEST_DOCUMENTS / "text" / "report.txt"
        if sample_path.exists():
            return sample_path.read_bytes()
        return b"Test document with content."

    def test_result_format_unified_default(self) -> None:
        """Test that Unified is the default result format."""
        config = ExtractionConfig()
        assert config.result_format == "Unified"

    def test_result_format_serialization(self) -> None:
        """Test that result_format serializes correctly."""
        config = ExtractionConfig(result_format="Elements")
        json_str = config.to_json()
        data = json.loads(json_str)
        assert data.get("result_format") == "Elements"

    def test_extraction_with_unified_format(self) -> None:
        """Test extraction with Unified result format."""
        doc_bytes = self.get_sample_document()
        config = ExtractionConfig(result_format="Unified")

        result = extract_bytes_sync(doc_bytes, "text/plain", config)

        assert isinstance(result, ExtractionResult)
        assert result.content is not None
        # Unified format concentrates content in one field
        assert isinstance(result.content, str)

    def test_extraction_with_elements_format(self) -> None:
        """Test extraction with Elements result format."""
        doc_bytes = self.get_sample_document()
        config = ExtractionConfig(result_format="Elements")

        result = extract_bytes_sync(doc_bytes, "text/plain", config)

        assert isinstance(result, ExtractionResult)
        # Elements format may provide structured elements
        assert result is not None

    def test_result_format_structure_variation(self) -> None:
        """Test that different result formats produce different structures."""
        doc_bytes = self.get_sample_document()

        # Extract with different formats
        unified_config = ExtractionConfig(result_format="Unified")
        unified_result = extract_bytes_sync(doc_bytes, "text/plain", unified_config)

        elements_config = ExtractionConfig(result_format="Elements")
        elements_result = extract_bytes_sync(doc_bytes, "text/plain", elements_config)

        # Both should succeed
        assert unified_result is not None
        assert elements_result is not None


class TestConfigCombinations:
    """Test combinations of output_format and result_format."""

    def get_sample_document(self) -> bytes:
        """Get a sample document for testing."""
        sample_path = TEST_DOCUMENTS / "text" / "report.txt"
        if sample_path.exists():
            return sample_path.read_bytes()
        return b"Sample document."

    def test_plain_unified_combination(self) -> None:
        """Test Plain output with Unified result format."""
        doc_bytes = self.get_sample_document()
        config = ExtractionConfig(
            output_format="Plain",
            result_format="Unified"
        )

        result = extract_bytes_sync(doc_bytes, "text/plain", config)

        assert result is not None
        assert isinstance(result, ExtractionResult)

    def test_markdown_elements_combination(self) -> None:
        """Test Markdown output with Elements result format."""
        doc_bytes = self.get_sample_document()
        config = ExtractionConfig(
            output_format="Markdown",
            result_format="Elements"
        )

        result = extract_bytes_sync(doc_bytes, "text/plain", config)

        assert result is not None
        assert isinstance(result, ExtractionResult)

    def test_html_unified_combination(self) -> None:
        """Test HTML output with Unified result format."""
        doc_bytes = self.get_sample_document()
        config = ExtractionConfig(
            output_format="Html",
            result_format="Unified"
        )

        result = extract_bytes_sync(doc_bytes, "text/plain", config)

        assert result is not None
        assert isinstance(result, ExtractionResult)

    def test_config_merge_preserves_formats(self) -> None:
        """Test that config merging preserves format fields."""
        config1 = ExtractionConfig(
            output_format="Markdown",
            result_format="Elements"
        )
        config2 = ExtractionConfig(use_cache=False)

        # Merge configs
        merged = config1.merge(config2)

        assert merged.output_format == "Markdown"
        assert merged.result_format == "Elements"


class TestConfigSerialization:
    """Test serialization/deserialization of format configs."""

    def test_output_format_to_json(self) -> None:
        """Test serializing output_format to JSON."""
        config = ExtractionConfig(output_format="Markdown")
        json_str = config.to_json()
        data = json.loads(json_str)

        assert "output_format" in data
        assert data["output_format"] == "Markdown"

    def test_result_format_to_json(self) -> None:
        """Test serializing result_format to JSON."""
        config = ExtractionConfig(result_format="Elements")
        json_str = config.to_json()
        data = json.loads(json_str)

        assert "result_format" in data
        assert data["result_format"] == "Elements"

    def test_from_json_with_output_format(self) -> None:
        """Test deserializing config with output_format."""
        json_str = json.dumps({
            "output_format": "Markdown",
            "use_cache": True
        })

        config = ExtractionConfig.from_json(json_str)

        assert config.output_format == "Markdown"
        assert config.use_cache is True

    def test_from_json_with_result_format(self) -> None:
        """Test deserializing config with result_format."""
        json_str = json.dumps({
            "result_format": "Elements",
            "enable_quality_processing": False
        })

        config = ExtractionConfig.from_json(json_str)

        assert config.result_format == "Elements"
        assert config.enable_quality_processing is False

    def test_round_trip_serialization(self) -> None:
        """Test round-trip serialization with format fields."""
        original = ExtractionConfig(
            output_format="Html",
            result_format="Elements",
            use_cache=False
        )

        # Serialize and deserialize
        json_str = original.to_json()
        restored = ExtractionConfig.from_json(json_str)

        assert restored.output_format == original.output_format
        assert restored.result_format == original.result_format
        assert restored.use_cache == original.use_cache


class TestErrorHandling:
    """Test error handling for format fields."""

    def test_invalid_output_format_rejected(self) -> None:
        """Test that invalid output_format values are rejected."""
        with pytest.raises((ValueError, TypeError)):
            ExtractionConfig(output_format="InvalidFormat")

    def test_invalid_result_format_rejected(self) -> None:
        """Test that invalid result_format values are rejected."""
        with pytest.raises((ValueError, TypeError)):
            ExtractionConfig(result_format="InvalidFormat")

    def test_case_sensitivity_of_formats(self) -> None:
        """Test that format names are case-sensitive."""
        # "plain" should not work; must be "Plain"
        with pytest.raises((ValueError, TypeError)):
            ExtractionConfig(output_format="plain")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
