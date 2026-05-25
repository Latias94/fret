#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ImUiLabelParts<'a> {
    pub visible: &'a str,
    pub identity: &'a str,
}

pub(super) fn parse_label_identity(label: &str) -> ImUiLabelParts<'_> {
    let visible_end = label.find("##").unwrap_or(label.len());
    let visible = &label[..visible_end];
    let identity = label
        .find("###")
        .map(|marker| &label[marker + 3..])
        .unwrap_or(label);

    ImUiLabelParts { visible, identity }
}

#[cfg(test)]
mod tests;
