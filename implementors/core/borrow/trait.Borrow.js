(function() {var implementors = {};
implementors["bytes"] = [{"text":"impl Borrow&lt;[u8]&gt; for Bytes","synthetic":false,"types":[]},{"text":"impl Borrow&lt;[u8]&gt; for BytesMut","synthetic":false,"types":[]}];
implementors["generic_array"] = [{"text":"impl&lt;T, N&gt; Borrow&lt;[T]&gt; for GenericArray&lt;T, N&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;N: ArrayLength&lt;T&gt;,&nbsp;</span>","synthetic":false,"types":[]}];
implementors["serde_bytes"] = [{"text":"impl Borrow&lt;Bytes&gt; for ByteBuf","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()