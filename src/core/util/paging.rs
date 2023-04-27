use base64::{engine::general_purpose, Engine as _};
use std::ops::Deref;

pub struct PageInfos {
    pub after: Option<String>,
    has_next_page: bool,
}

impl PageInfos {
    pub fn page_after(cursor: i64, has_next_page: bool) -> PageInfos {
        PageInfos {
            after: Some(general_purpose::STANDARD.encode(format!("{}", cursor))),
            has_next_page,
        }
    }

    pub fn no_page_after() -> PageInfos {
        PageInfos {
            after: None,
            has_next_page: false,
        }
    }

    pub fn after(&self) -> Option<String> {
        self.after.clone()
    }

    pub fn has_next_page(&self) -> bool {
        self.has_next_page
    }
}

pub struct Page<T> {
    pub items: Vec<T>,
    pub page_infos: PageInfos,
}

#[derive(Debug, Clone)]
pub struct Paging {
    pub first: i64,
    pub after: Option<String>,
}

impl Paging {
    pub fn new(first: i64, after: Option<String>) -> Paging {
        Paging { first, after }
    }
}

pub struct IntCursor(i64);

impl Deref for IntCursor {
    type Target = i64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Paging> for IntCursor {
    fn from(paging: Paging) -> IntCursor {
        if let Some(after) = &paging.after {
            let decoded = general_purpose::STANDARD
                .decode(after)
                .map(|v| String::from_utf8(v).unwrap_or("".to_string()))
                .unwrap_or("".to_string());
            let raw: Vec<&str> = decoded.split(";").collect();
            let cursor = raw
                .get(0)
                .map(|s| s.to_string().parse::<i64>().unwrap_or(0))
                .unwrap_or(0);
            IntCursor(cursor)
        } else {
            IntCursor(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_infos_should_encode_after_cursor_case_1() {
        let page_infos = PageInfos::page_after(17, true);

        assert_eq!(&page_infos.has_next_page(), &true);
        assert_eq!(&page_infos.after(), &Some("MTc=".to_string()));
    }

    #[test]
    fn page_infos_should_encode_after_cursor_case_2() {
        let page_infos = PageInfos::page_after(23, false);

        assert_eq!(&page_infos.has_next_page(), &false);
        assert_eq!(&page_infos.after(), &Some("MjM=".to_string()));
    }
    #[test]
    fn no_page_after() {
        let page_infos = PageInfos::no_page_after();

        assert_eq!(&page_infos.has_next_page(), &false);
        assert_eq!(&page_infos.after(), &None);
    }

    #[test]
    fn int_cursor_roundtrip_from_page_to_paging_to_cursor() {
        let page_infos = PageInfos::page_after(17, true);
        let paging = Paging::new(5, page_infos.after);
        let cursor: IntCursor = paging.into();

        assert_eq!(cursor.deref(), &17_i64);
    }

    #[test]
    fn int_cursor_from_paging_with_none_as_cursor() {
        let paging = Paging::new(5, None);
        let cursor: IntCursor = paging.into();

        assert_eq!(cursor.deref(), &0_i64);
    }

    #[test]
    fn int_cursor_from_paging_with_invalid_base64() {
        let paging = Paging::new(5, Some("yuk".to_string()));
        let cursor: IntCursor = paging.into();

        assert_eq!(cursor.deref(), &0_i64);
    }

    #[test]
    fn int_cursor_from_paging_with_base64_cursor() {
        let paging = Paging::new(5, Some(general_purpose::STANDARD.encode("487")));
        let cursor: IntCursor = paging.into();

        assert_eq!(cursor.deref(), &487_i64);
    }

    #[test]
    fn int_cursor_from_paging_with_base64_cursor_and_extra_data() {
        let paging = Paging::new(
            5,
            Some(general_purpose::STANDARD.encode("467;firstname;463")),
        );
        let cursor: IntCursor = paging.into();

        assert_eq!(cursor.deref(), &467_i64);
    }
}
