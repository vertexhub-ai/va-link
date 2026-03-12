use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use uuid::Uuid;

use crate::models::{Link, New