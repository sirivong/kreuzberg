# Benchmarks {: style="display: none;" }

<div class="full-width-iframe" markdown="1">
<iframe src="/benchmarks/app/"
        width="100%"
        height="100vh"
        frameborder="0"
        sandbox="allow-same-origin allow-scripts allow-popups allow-forms"
        style="border: none; display: block; min-height: calc(100vh - 64px); height: calc(100vh - 64px);"
        title="Interactive Benchmark Visualizer">
  <p>Your browser does not support iframes. Please visit the <a href="/benchmarks/app/">interactive benchmark visualizer</a> directly.</p>
</iframe>

<script>
(function() {
  const iframe = document.querySelector('iframe[title="Interactive Benchmark Visualizer"]');
  if (!iframe) return;

  const sendTheme = () => {
    const scheme = document.body.getAttribute('data-md-color-scheme');
    const isDark = scheme === 'slate';
    iframe.contentWindow.postMessage({
      type: 'theme',
      value: isDark ? 'dark' : 'light'
    }, '*');
  };

  iframe.addEventListener('load', () => setTimeout(sendTheme, 100));
  if (iframe.contentDocument && iframe.contentDocument.readyState === 'complete') {
    setTimeout(sendTheme, 100);
  }

  const observer = new MutationObserver(sendTheme);
  observer.observe(document.body, {
    attributes: true,
    attributeFilter: ['data-md-color-scheme']
  });
})();
</script>
</div>
