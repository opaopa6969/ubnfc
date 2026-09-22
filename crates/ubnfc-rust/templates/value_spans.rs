// value span の path は共有バッファに push / truncate で組み立てる（field ごとの format! を避ける）。
trait CollectValueSpans {
    fn collect(
        &self,
        path: &mut String,
        texts: &TextSpans,
        out: &mut Vec<ValueSpan>,
    );
}
impl CollectValueSpans for String {
    fn collect(
        &self,
        path: &mut String,
        texts: &TextSpans,
        out: &mut Vec<ValueSpan>,
    ) {
        if let Some(span) = texts.get(&(self.as_ptr() as usize)) {
            out.push(ValueSpan {
                path: path.clone(),
                span: *span,
                text: None,
            });
        }
    }
}
impl<T: CollectValueSpans> CollectValueSpans for Vec<T> {
    fn collect(
        &self,
        path: &mut String,
        texts: &TextSpans,
        out: &mut Vec<ValueSpan>,
    ) {
        for (i, value) in self.iter().enumerate() {
            let len = path.len();
            path.push('/');
            path.push_str(&i.to_string());
            value.collect(path, texts, out);
            path.truncate(len);
        }
    }
}
impl<T: CollectValueSpans> CollectValueSpans for Option<T> {
    fn collect(
        &self,
        path: &mut String,
        texts: &TextSpans,
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
        path: &mut String,
        texts: &TextSpans,
        out: &mut Vec<ValueSpan>,
    ) {
        self.as_ref().collect(path, texts, out);
    }
}
