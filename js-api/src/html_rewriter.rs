use super::handlers::{DocumentContentHandlers, ElementContentHandlers};
use super::*;
use js_sys::{Function as JsFunction, Uint8Array};
use lol_html::{
    DocumentContentHandlers as NativeDocumentContentHandlers,
    ElementContentHandlers as NativeElementContentHandlers, HtmlRewriter as NativeHTMLRewriter,
    OutputSink, Selector, Settings,
};

struct JsOutputSink(JsFunction);

impl JsOutputSink {
    fn new(func: &JsFunction) -> Self {
        JsOutputSink(func.clone())
    }
}

impl OutputSink for JsOutputSink {
    #[inline]
    fn handle_chunk(&mut self, chunk: &[u8]) {
        let this = JsValue::NULL;
        let chunk = Uint8Array::from(chunk);

        // NOTE: the error is handled in the JS wrapper.
        self.0.call1(&this, &chunk).unwrap();
    }
}

#[wasm_bindgen]
#[derive(Default)]
pub struct HTMLRewriter {
    selectors: Vec<Selector>,
    element_content_handlers: Vec<NativeElementContentHandlers<'static>>,
    document_content_handlers: Vec<NativeDocumentContentHandlers<'static>>,
    output_sink: Option<JsOutputSink>,
    encoding: String,
    inner: Option<NativeHTMLRewriter<'static, JsOutputSink>>,
}

#[wasm_bindgen]
impl HTMLRewriter {
    #[wasm_bindgen(constructor)]
    pub fn new(encoding: String, output_sink: &JsFunction) -> Self {
        HTMLRewriter {
            output_sink: Some(JsOutputSink::new(output_sink)),
            encoding,
            ..Self::default()
        }
    }

    fn assert_not_fully_constructed(&self) -> JsResult<()> {
        if self.inner.is_some() {
            Err("Handlers can't be added after write.".into())
        } else {
            Ok(())
        }
    }

    fn ensure_inner(&mut self) -> JsResult<()> {
        if self.inner.is_none() {
            let output_sink = self.output_sink.take().unwrap();
            // NOTE: selector are passed by reference to the rewriter ctor, though they
            // are not stored in the rewriter, so we need their references to be valid
            // only during the rewriter invocation.
            let selectors: Vec<_> = self.selectors.drain(..).collect();

            let settings = Settings {
                element_content_handlers: self
                    .element_content_handlers
                    .drain(..)
                    .enumerate()
                    .map(|(i, h)| (&selectors[i], h))
                    .collect(),

                document_content_handlers: self.document_content_handlers.drain(..).collect(),
                encoding: &self.encoding,
                ..Settings::default()
            };

            let rewriter = NativeHTMLRewriter::try_new(settings, output_sink).into_js_result()?;

            self.inner = Some(rewriter);
        }

        Ok(())
    }

    pub fn on(&mut self, selector: &str, handlers: ElementContentHandlers) -> JsResult<()> {
        self.assert_not_fully_constructed()?;

        let selector = selector.parse::<Selector>().into_js_result()?;

        self.selectors.push(selector);
        self.element_content_handlers.push(handlers.into_native());

        Ok(())
    }

    #[wasm_bindgen(method, js_name=onDocument)]
    pub fn on_document(&mut self, handlers: DocumentContentHandlers) -> JsResult<()> {
        self.assert_not_fully_constructed()?;
        self.document_content_handlers.push(handlers.into_native());

        Ok(())
    }
}
