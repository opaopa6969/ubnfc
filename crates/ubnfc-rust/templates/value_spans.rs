trait CollectValueSpans {
    fn collect(
        &self,
        path: &str,
        texts: &std::collections::HashMap<usize, Span>,
        out: &mut Vec<ValueSpan>,
    );
}
impl CollectValueSpans for String {
    fn collect(
        &self,
        path: &str,
        texts: &std::collections::HashMap<usize, Span>,
        out: &mut Vec<ValueSpan>,
    ) {
        if let Some(span) = texts.get(&(self.as_ptr() as usize)) {
            out.push(ValueSpan {
                path: path.into(),
                span: *span,
                text: None,
            });
        }
    }
}
impl<T: CollectValueSpans> CollectValueSpans for Vec<T> {
    fn collect(
        &self,
        path: &str,
        texts: &std::collections::HashMap<usize, Span>,
        out: &mut Vec<ValueSpan>,
    ) {
        for (i, value) in self.iter().enumerate() {
            value.collect(&format!("{path}/{i}"), texts, out);
        }
    }
}
impl<T: CollectValueSpans> CollectValueSpans for Option<T> {
    fn collect(
        &self,
        path: &str,
        texts: &std::collections::HashMap<usize, Span>,
        out: &mut Vec<ValueSpan>,
    ) {
        if let Some(value) = self {
            value.collect(path, texts, out);
        }
    }
}
impl<T: CollectValueSpans> CollectValueSpans for Box<T> {
    fn collect(
        &self,
        path: &str,
        texts: &std::collections::HashMap<usize, Span>,
        out: &mut Vec<ValueSpan>,
    ) {
        self.as_ref().collect(path, texts, out);
    }
}
