# Analysis Report: mio_gemini-2.5-flash-nothinking_20251127_012706

## 1. Executive Summary
- Total Samples: 224
- Success: 4 (1.8%)
- Failures: 220 (98.2%)

## 2. Failure Distribution
| Category | Count | Percentage |
|---|---|---|
| RUSTC_ERROR | 167 | 75.9% |
| OTHER_FAILURE | 47 | 21.4% |
| UNSTABLE_FEATURE(E0658) | 4 | 1.8% |
| TRUNCATED_BRACES | 2 | 0.9% |

## 3. Top Rust Error Codes
| Code | Count | Description |
|---|---|---|
| E0599 | 117 | Method/field not found |
| E0433 | 76 | Failed to resolve import |
| E0432 | 60 | Unresolved import |
| E0603 | 9 |  |
| E0425 | 8 | Unresolved name |
| E0061 | 7 | Wrong number of function arguments |
| E0277 | 4 | Trait not implemented |
| E0658 | 4 | Unstable feature |
| E0308 | 4 | Type mismatch |
| E0071 | 1 |  |

## 4. Truncation Issues
| Type | Count | Percentage |
|---|---|---|
| TRUNCATED_BRACES | 2 | 0.9% |

## 5. Details
| Node ID | Category | Reason |
|---|---|---|
| sys::shell::selector::event::token | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| sys::shell::selector::event::token | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| sys::shell::selector::event::is_readable | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| sys::shell::selector::event::is_readable | RUSTC_ERROR | Failed to resolve import: E0433 |
| sys::shell::selector::event::is_writable | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::selector::event::is_writable | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| sys::shell::selector::event::is_error | RUSTC_ERROR | Method/field not found: E0599 |
| sys::shell::selector::event::is_error | RUSTC_ERROR | Failed to resolve import: E0433 |
| sys::shell::selector::event::is_read_closed | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| sys::shell::selector::event::is_read_closed | RUSTC_ERROR | Method/field not found: E0603, E0433, E0599 |
| sys::shell::selector::event::is_write_closed | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| sys::shell::selector::event::is_write_closed | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| sys::shell::selector::event::is_priority | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::selector::event::is_priority | RUSTC_ERROR | Method/field not found: E0425, E0433, E0599 |
| sys::shell::selector::event::is_aio | RUSTC_ERROR | Method/field not found: E0603, E0599 |
| sys::shell::selector::event::is_aio | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| sys::shell::selector::event::is_lio | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| sys::shell::selector::event::is_lio | RUSTC_ERROR | Method/field not found: E0603, E0433, E0599 |
| sys::shell::selector::event::debug_details | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::selector::event::debug_details | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| sys::shell::selector::event::debug_details | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| sys::shell::selector::event::debug_details | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| sys::shell::selector::event::debug_details | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| sys::shell::selector::Selector::try_clone | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::selector::Selector::try_clone | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::selector::Selector::try_clone | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::selector::Selector::try_clone | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::selector::Selector::try_clone | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::selector::Selector::try_clone | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::selector::Selector::try_clone | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::selector::Selector::select | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::selector::Selector::select | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::selector::Selector::select | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::selector::Selector::select | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::selector::Selector::select | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::selector::Selector::select | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::selector::Selector::select | RUSTC_ERROR | Compiler errors: E0061 |
| sys::shell::selector::Selector::select | RUSTC_ERROR | Method/field not found: E0599 |
| sys::shell::selector::Selector::select | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| sys::shell::selector::Selector::select | RUSTC_ERROR | Type mismatch errors: E0308, E0599 |
| sys::shell::selector::Selector::register_waker | RUSTC_ERROR | Failed to resolve import: E0433 |
| sys::shell::selector::Selector::register_waker | RUSTC_ERROR | Method/field not found: E0599 |
| sys::shell::selector::Selector::register_waker | RUSTC_ERROR | Failed to resolve import: E0433 |
| sys::shell::selector::Selector::register_waker | RUSTC_ERROR | Compiler errors: E0061 |
| <sys::shell::selector::Selector as std::os::fd::AsRawFd>::as_raw_fd | RUSTC_ERROR | Method/field not found: E0599 |
| <sys::shell::selector::Selector as std::os::fd::AsRawFd>::as_raw_fd | RUSTC_ERROR | Compiler errors: E0061 |
| <std::boxed::Box<T> as event::source::Source>::register | OTHER_FAILURE | Compilation failed without specific error code |
| <std::boxed::Box<T> as event::source::Source>::register | RUSTC_ERROR | Compiler errors: E0432 |
| <std::boxed::Box<T> as event::source::Source>::register | OTHER_FAILURE | Compilation failed without specific error code |
| <std::boxed::Box<T> as event::source::Source>::register | RUSTC_ERROR | Method/field not found: E0599 |
| <std::boxed::Box<T> as event::source::Source>::register | OTHER_FAILURE | Compilation failed without specific error code |
| <std::boxed::Box<T> as event::source::Source>::register | OTHER_FAILURE | Compilation failed without specific error code |
| <std::boxed::Box<T> as event::source::Source>::register | OTHER_FAILURE | Compilation failed without specific error code |
| <std::boxed::Box<T> as event::source::Source>::register | OTHER_FAILURE | Compilation failed without specific error code |
| <std::boxed::Box<T> as event::source::Source>::register | RUSTC_ERROR | Method/field not found: E0599 |
| <std::boxed::Box<T> as event::source::Source>::register | OTHER_FAILURE | Compilation failed without specific error code |
| <std::boxed::Box<T> as event::source::Source>::register | OTHER_FAILURE | Compilation failed without specific error code |
| <std::boxed::Box<T> as event::source::Source>::register | RUSTC_ERROR | Method/field not found: E0599 |
| <std::boxed::Box<T> as event::source::Source>::register | RUSTC_ERROR | Type mismatch errors: E0308 |
| <std::boxed::Box<T> as event::source::Source>::register | RUSTC_ERROR | Type mismatch errors: E0308, E0599 |
| <std::boxed::Box<T> as event::source::Source>::reregister | RUSTC_ERROR | Method/field not found: E0599 |
| <std::boxed::Box<T> as event::source::Source>::reregister | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| <std::boxed::Box<T> as event::source::Source>::deregister | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| <std::boxed::Box<T> as event::source::Source>::deregister | RUSTC_ERROR | Compiler errors: E0277, E0432 |
| interest::Interest::add | RUSTC_ERROR | Failed to resolve import: E0433 |
| interest::Interest::add | RUSTC_ERROR | Method/field not found: E0599 |
| interest::Interest::add | RUSTC_ERROR | Method/field not found: E0599 |
| interest::Interest::add | RUSTC_ERROR | Method/field not found: E0599 |
| interest::Interest::is_readable | RUSTC_ERROR | Failed to resolve import: E0433 |
| interest::Interest::is_readable | OTHER_FAILURE | Compilation failed without specific error code |
| interest::Interest::is_readable | RUSTC_ERROR | Failed to resolve import: E0433 |
| interest::Interest::is_readable | RUSTC_ERROR | Method/field not found: E0599 |
| interest::Interest::is_lio | RUSTC_ERROR | Method/field not found: E0599 |
| interest::Interest::is_lio | RUSTC_ERROR | Method/field not found: E0599 |
| interest::Interest::is_lio | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| interest::Interest::is_lio | RUSTC_ERROR | Method/field not found: E0599 |
| <interest::Interest as std::ops::BitOr>::bitor | OTHER_FAILURE | Compilation failed without specific error code |
| <interest::Interest as std::ops::BitOr>::bitor | OTHER_FAILURE | Compilation failed without specific error code |
| <interest::Interest as std::ops::BitOrAssign>::bitor_assign | RUSTC_ERROR | Method/field not found: E0599 |
| <interest::Interest as std::ops::BitOrAssign>::bitor_assign | OTHER_FAILURE | Compilation failed without specific error code |
| poll::Poll::registry | RUSTC_ERROR | Failed to resolve import: E0433 |
| poll::Poll::registry | RUSTC_ERROR | Failed to resolve import: E0433 |
| poll::Poll::registry | RUSTC_ERROR | Failed to resolve import: E0433 |
| poll::Poll::registry | RUSTC_ERROR | Method/field not found: E0599 |
| poll::Poll::registry | RUSTC_ERROR | Method/field not found: E0599 |
| poll::Poll::registry | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| poll::Poll::registry | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| poll::Poll::poll | OTHER_FAILURE | Compilation failed without specific error code |
| poll::Poll::poll | OTHER_FAILURE | Compilation failed without specific error code |
| poll::Poll::poll | OTHER_FAILURE | Compilation failed without specific error code |
| poll::Poll::poll | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| poll::Poll::poll | RUSTC_ERROR | Method/field not found: E0603, E0599 |
| poll::Poll::poll | RUSTC_ERROR | Method/field not found: E0599 |
| poll::Poll::poll | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| <poll::Poll as std::os::fd::AsRawFd>::as_raw_fd | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| <poll::Poll as std::os::fd::AsRawFd>::as_raw_fd | RUSTC_ERROR | Method/field not found: E0599 |
| poll::Registry::register | OTHER_FAILURE | Compilation failed without specific error code |
| poll::Registry::register | OTHER_FAILURE | Compilation failed without specific error code |
| poll::Registry::register | OTHER_FAILURE | Compilation failed without specific error code |
| poll::Registry::register | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| poll::Registry::register | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| poll::Registry::register | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| poll::Registry::register | RUSTC_ERROR | Method/field not found: E0277, E0599 |
| poll::Registry::reregister | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| poll::Registry::reregister | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| poll::Registry::reregister | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| poll::Registry::reregister | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| poll::Registry::deregister | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| poll::Registry::deregister | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| poll::Registry::deregister | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| poll::Registry::deregister | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| poll::Registry::try_clone | RUSTC_ERROR | Method/field not found: E0599 |
| poll::Registry::try_clone | RUSTC_ERROR | Method/field not found: E0599 |
| poll::Registry::try_clone | RUSTC_ERROR | Method/field not found: E0599 |
| poll::Registry::try_clone | RUSTC_ERROR | Method/field not found: E0599 |
| poll::Registry::register_waker | RUSTC_ERROR | Method/field not found: E0599 |
| poll::Registry::register_waker | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| poll::Registry::register_waker | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| poll::Registry::register_waker | RUSTC_ERROR | Method/field not found: E0599 |
| poll::Registry::selector | RUSTC_ERROR | Method/field not found: E0599 |
| poll::Registry::selector | RUSTC_ERROR | Method/field not found: E0599 |
| poll::Registry::selector | RUSTC_ERROR | Method/field not found: E0599 |
| poll::Registry::selector | RUSTC_ERROR | Method/field not found: E0599 |
| <poll::Registry as std::os::fd::AsRawFd>::as_raw_fd | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| <poll::Registry as std::os::fd::AsRawFd>::as_raw_fd | RUSTC_ERROR | Compiler errors: E0432 |
| sys::shell::waker::Waker::new | RUSTC_ERROR | Compiler errors: E0061 |
| sys::shell::waker::Waker::new | RUSTC_ERROR | Compiler errors: E0061 |
| sys::shell::waker::Waker::new | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| sys::shell::waker::Waker::wake | RUSTC_ERROR | Failed to resolve import: E0433 |
| sys::shell::waker::Waker::wake | RUSTC_ERROR | Compiler errors: E0432 |
| sys::shell::waker::Waker::wake | RUSTC_ERROR | Compiler errors: E0432 |
| sys::shell::waker::Waker::wake | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::waker::Waker::wake | OTHER_FAILURE | Compilation failed without specific error code |
| sys::shell::waker::Waker::wake | RUSTC_ERROR | Failed to resolve import: E0433 |
| sys::shell::waker::Waker::wake | RUSTC_ERROR | Compiler errors: E0423 |
| token::<impl std::convert::From<token::Token> for usize>::from | OTHER_FAILURE | Compilation failed without specific error code |
| waker::Waker::new | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| waker::Waker::new | RUSTC_ERROR | Method/field not found: E0599 |
| waker::Waker::new | RUSTC_ERROR | Method/field not found: E0599 |
| waker::Waker::wake | RUSTC_ERROR | Compiler errors: E0432 |
| waker::Waker::wake | RUSTC_ERROR | Compiler errors: E0432 |
| waker::Waker::wake | RUSTC_ERROR | Method/field not found: E0599 |
| waker::Waker::wake | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| waker::Waker::wake | RUSTC_ERROR | Compiler errors: E0432 |
| waker::Waker::wake | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| waker::Waker::wake | RUSTC_ERROR | Compiler errors: E0432 |
| event::event::Event::token | RUSTC_ERROR | Method/field not found: E0599 |
| event::event::Event::token | RUSTC_ERROR | Method/field not found: E0599 |
| event::event::Event::token | RUSTC_ERROR | Method/field not found: E0599 |
| event::event::Event::token | TRUNCATED_BRACES | Unbalanced braces: { (6) vs } (4) |
| event::event::Event::token | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| event::event::Event::token | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| event::event::Event::token | TRUNCATED_BRACES | Unbalanced braces: { (3) vs } (0) |
| event::event::Event::is_readable | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| event::event::Event::is_readable | RUSTC_ERROR | Method/field not found: E0425, E0433, E0599 |
| event::event::Event::is_readable | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| event::event::Event::is_readable | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| event::event::Event::is_writable | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| event::event::Event::is_writable | RUSTC_ERROR | Method/field not found: E0425, E0433, E0599 |
| event::event::Event::is_writable | RUSTC_ERROR | Failed to resolve import: E0433 |
| event::event::Event::is_writable | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| event::event::Event::is_error | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| event::event::Event::is_error | RUSTC_ERROR | Method/field not found: E0603, E0432, E0599 |
| event::event::Event::is_error | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| event::event::Event::is_error | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| event::event::Event::is_read_closed | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| event::event::Event::is_read_closed | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| event::event::Event::is_read_closed | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| event::event::Event::is_read_closed | RUSTC_ERROR | Method/field not found: E0603, E0599 |
| event::event::Event::is_write_closed | RUSTC_ERROR | Method/field not found: E0603, E0432, E0599 |
| event::event::Event::is_write_closed | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| event::event::Event::is_write_closed | RUSTC_ERROR | Failed to resolve import: E0433 |
| event::event::Event::is_write_closed | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| event::event::Event::is_priority | RUSTC_ERROR | Failed to resolve import: E0433 |
| event::event::Event::is_priority | RUSTC_ERROR | Method/field not found: E0425, E0433, E0599 |
| event::event::Event::is_priority | RUSTC_ERROR | Method/field not found: E0603, E0432, E0599 |
| event::event::Event::is_priority | RUSTC_ERROR | Method/field not found: E0425, E0433, E0599 |
| event::event::Event::is_aio | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| event::event::Event::is_aio | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| event::event::Event::is_aio | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| event::event::Event::is_aio | RUSTC_ERROR | Compiler errors: E0432 |
| event::event::Event::is_lio | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| event::event::Event::is_lio | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| event::event::Event::is_lio | RUSTC_ERROR | Method/field not found: E0603, E0432, E0599 |
| event::event::Event::is_lio | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| event::event::Event::from_sys_event_ref | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| event::event::Event::from_sys_event_ref | RUSTC_ERROR | Method/field not found: E0599 |
| event::event::Event::from_sys_event_ref | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| event::events::Events::with_capacity | RUSTC_ERROR | Failed to resolve import: E0433 |
| event::events::Events::with_capacity | OTHER_FAILURE | Compilation failed without specific error code |
| event::events::Events::with_capacity | RUSTC_ERROR | Failed to resolve import: E0433 |
| event::events::Events::capacity | RUSTC_ERROR | Failed to resolve import: E0433 |
| event::events::Events::capacity | OTHER_FAILURE | Compilation failed without specific error code |
| event::events::Events::is_empty | RUSTC_ERROR | Compiler errors: E0432 |
| event::events::Events::is_empty | OTHER_FAILURE | Compilation failed without specific error code |
| event::events::Events::is_empty | RUSTC_ERROR | Method/field not found: E0425, E0433, E0599 |
| event::events::Events::is_empty | OTHER_FAILURE | Compilation failed without specific error code |
| event::events::Events::iter | OTHER_FAILURE | Compilation failed without specific error code |
| event::events::Events::iter | RUSTC_ERROR | Compiler errors: E0369 |
| event::events::Events::iter | RUSTC_ERROR | Compiler errors: E0432 |
| event::events::Events::iter | RUSTC_ERROR | Compiler errors: E0432 |
| event::events::Events::clear | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| event::events::Events::clear | RUSTC_ERROR | Compiler errors: E0432 |
| event::events::Events::clear | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| event::events::Events::clear | RUSTC_ERROR | Compiler errors: E0432 |
| event::events::Events::sys | OTHER_FAILURE | Compilation failed without specific error code |
| event::events::Events::sys | OTHER_FAILURE | Compilation failed without specific error code |
| event::events::Events::sys | OTHER_FAILURE | Compilation failed without specific error code |
| event::events::Events::sys | OTHER_FAILURE | Compilation failed without specific error code |
| <&'a event::events::Events as std::iter::IntoIterator>::into_iter | RUSTC_ERROR | Failed to resolve import: E0433 |
| <&'a event::events::Events as std::iter::IntoIterator>::into_iter | RUSTC_ERROR | Failed to resolve import: E0433 |
| <event::events::Iter<'a> as std::iter::Iterator>::next | RUSTC_ERROR | Failed to resolve import: E0433 |
| <event::events::Iter<'a> as std::iter::Iterator>::next | RUSTC_ERROR | Failed to resolve import: E0433 |
| <event::events::Iter<'a> as std::iter::Iterator>::next | RUSTC_ERROR | Failed to resolve import: E0433 |
| <event::events::Iter<'a> as std::iter::Iterator>::next | RUSTC_ERROR | Type mismatch errors: E0432, E0425, E0308, E0560 |
| <event::events::Iter<'a> as std::iter::Iterator>::next | RUSTC_ERROR | Method/field not found: E0599 |
| <event::events::Iter<'a> as std::iter::Iterator>::size_hint | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| <event::events::Iter<'a> as std::iter::Iterator>::size_hint | RUSTC_ERROR | Method/field not found: E0599 |
| <event::events::Iter<'a> as std::iter::Iterator>::count | RUSTC_ERROR | Failed to resolve import: E0433 |
| <event::events::Iter<'a> as std::iter::Iterator>::count | OTHER_FAILURE | Compilation failed without specific error code |
