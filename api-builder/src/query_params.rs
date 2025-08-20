use core::ops::{Deref, DerefMut};
use std::borrow::Cow;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct QueryParamPair {
    pub key: Cow<'static, str>,
    pub value: Cow<'static, str>,
}
impl QueryParamPair {
    pub fn new<K, V>(key: K, value: V) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}
impl<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>> From<(K, V)> for QueryParamPair {
    fn from(pair: (K, V)) -> Self {
        Self {
            key: pair.0.into(),
            value: pair.1.into(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct QueryParamPairs(pub Vec<QueryParamPair>);
impl QueryParamPairs {
    pub fn append<T: Into<QueryParamPairs>>(&mut self, other: T) {
        self.0.append(&mut other.into());
    }

    pub fn push<T: Into<QueryParamPair>>(&mut self, value: T) {
        self.0.push(value.into());
    }

    pub fn push_hashmap<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>>(
        &mut self,
        name: &str,
        value: std::collections::HashMap<K, V>,
    ) {
        let mut entries: Vec<(Cow<'static, str>, Cow<'static, str>)> = value
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        for (key, value) in entries {
            self.push((format!("{}[{}]", name, key), value));
        }
    }
}
impl Deref for QueryParamPairs {
    type Target = Vec<QueryParamPair>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for QueryParamPairs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<Vec<QueryParamPair>> for QueryParamPairs {
    fn from(pairs: Vec<QueryParamPair>) -> Self {
        Self(pairs)
    }
}
impl<K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>> From<Vec<(K, V)>> for QueryParamPairs {
    fn from(value: Vec<(K, V)>) -> Self {
        Self(value.into_iter().map(|x| x.into()).collect())
    }
}

impl Extend<QueryParamPair> for QueryParamPairs {
    fn extend<T: IntoIterator<Item = QueryParamPair>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl FromIterator<QueryParamPair> for QueryParamPairs {
    fn from_iter<T: IntoIterator<Item = QueryParamPair>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
